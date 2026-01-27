//
//  SearchPanel.swift
//  Nova
//
//  NSPanel-based search interface that can appear over fullscreen apps.
//

import Cocoa

final class SearchPanel: NSPanel {
    private let searchField: NSTextField
    private let resultsTableView: NSTableView
    private let scrollView: NSScrollView
    private var divider: NSBox!

    private var results: [SearchResult] = []
    private var selectedIndex: Int = 0
    private(set) var isPanelVisible: Bool = false

    // Extension view support
    private var navigationStack: NavigationStack?
    private var isShowingExtension: Bool = false
    private var currentExtensionId: String?

    var onSearch: ((String) -> [SearchResult])?
    var onExecute: ((UInt32) -> ExecutionResult)?
    var onExecuteExtension: ((String, String, String?) -> ExtensionResponse?)?
    var onExtensionEvent: ((String, String, [String: Any]) -> ExtensionResponse?)?
    var onHide: (() -> Void)?

    // MARK: - Initialization

    override init(
        contentRect: NSRect,
        styleMask style: NSWindow.StyleMask,
        backing backingStoreType: NSWindow.BackingStoreType,
        defer flag: Bool
    ) {
        // Create search field
        searchField = NSTextField()
        searchField.placeholderString = "Search apps, commands, and more..."
        searchField.font = .systemFont(ofSize: 24, weight: .light)
        searchField.isBordered = false
        searchField.focusRingType = .none
        searchField.drawsBackground = false
        searchField.translatesAutoresizingMaskIntoConstraints = false

        // Create table view for results
        resultsTableView = NSTableView()
        resultsTableView.headerView = nil
        resultsTableView.backgroundColor = .clear
        resultsTableView.selectionHighlightStyle = .sourceList
        resultsTableView.rowHeight = 52
        resultsTableView.intercellSpacing = NSSize(width: 0, height: 2)
        resultsTableView.translatesAutoresizingMaskIntoConstraints = false
        resultsTableView.style = .plain
        resultsTableView.usesAlternatingRowBackgroundColors = false

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("result"))
        column.width = 600
        resultsTableView.addTableColumn(column)

        // Create scroll view
        scrollView = NSScrollView()
        scrollView.documentView = resultsTableView
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.drawsBackground = false
        scrollView.translatesAutoresizingMaskIntoConstraints = false

        super.init(
            contentRect: contentRect,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )

        setupPanel()
        setupLayout()
        setupDelegates()
    }

    // MARK: - Key Window Override

    // Allow panel to become key window and accept keyboard input
    override var canBecomeKey: Bool { true }
    override var canBecomeMain: Bool { true }

    // MARK: - Setup

    private func setupPanel() {
        // Panel configuration for fullscreen overlay
        level = .floating
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient]
        isOpaque = false
        backgroundColor = .clear
        hasShadow = true
        hidesOnDeactivate = false

        // Visual effect for translucent background
        let visualEffect = NSVisualEffectView()
        visualEffect.material = .hudWindow
        visualEffect.state = .active
        visualEffect.blendingMode = .behindWindow
        visualEffect.wantsLayer = true
        visualEffect.layer?.cornerRadius = 12
        visualEffect.layer?.masksToBounds = true
        visualEffect.translatesAutoresizingMaskIntoConstraints = false

        contentView = visualEffect
    }

    private func setupLayout() {
        guard let contentView = contentView else { return }

        // Divider between search field and results
        divider = NSBox()
        divider.boxType = .separator
        divider.translatesAutoresizingMaskIntoConstraints = false

        contentView.addSubview(searchField)
        contentView.addSubview(divider)
        contentView.addSubview(scrollView)

        NSLayoutConstraint.activate([
            // Search field
            searchField.topAnchor.constraint(equalTo: contentView.topAnchor, constant: 16),
            searchField.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 16),
            searchField.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -16),
            searchField.heightAnchor.constraint(equalToConstant: 36),

            // Divider
            divider.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: 12),
            divider.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: 8),
            divider.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -8),

            // Scroll view
            scrollView.topAnchor.constraint(equalTo: divider.bottomAnchor, constant: 8),
            scrollView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -8),
        ])
    }

    private func setupDelegates() {
        searchField.delegate = self
        resultsTableView.dataSource = self
        resultsTableView.delegate = self
    }

    // MARK: - Public API

    func show() {
        // Center on screen
        if let screen = NSScreen.main {
            let screenFrame = screen.frame
            let panelWidth: CGFloat = 620
            let panelHeight: CGFloat = 400

            let x = (screenFrame.width - panelWidth) / 2 + screenFrame.origin.x
            let y = (screenFrame.height - panelHeight) / 2 + screenFrame.origin.y + 100 // Slightly above center

            setFrame(NSRect(x: x, y: y, width: panelWidth, height: panelHeight), display: true)
        }

        makeKeyAndOrderFront(nil)
        searchField.stringValue = ""
        results = []
        selectedIndex = 0
        resultsTableView.reloadData()

        // Focus the search field
        makeFirstResponder(searchField)
        isPanelVisible = true
    }

    func hide() {
        isPanelVisible = false
        if isShowingExtension {
            hideExtensionView()
        }
        orderOut(nil)
        onHide?()
    }

    // MARK: - Keyboard Navigation

    override func keyDown(with event: NSEvent) {
        switch Int(event.keyCode) {
        case 53: // Escape
            if isShowingExtension {
                hideExtensionView()
            } else {
                hide()
            }

        case 125: // Down arrow
            if selectedIndex < results.count - 1 {
                selectedIndex += 1
                resultsTableView.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
                resultsTableView.scrollRowToVisible(selectedIndex)
            }

        case 126: // Up arrow
            if selectedIndex > 0 {
                selectedIndex -= 1
                resultsTableView.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
                resultsTableView.scrollRowToVisible(selectedIndex)
            }

        case 36: // Return
            executeSelected()

        default:
            super.keyDown(with: event)
        }
    }

    private func executeSelected() {
        guard !results.isEmpty, selectedIndex < results.count else { return }

        let selectedResult = results[selectedIndex]

        // Handle Deno extension commands specially
        switch selectedResult {
        case .denoCommand(let data):
            executeExtensionCommand(extensionId: data.extensionId, commandId: data.commandId, argument: nil)
            return
        case .denoCommandWithArg(let data):
            executeExtensionCommand(extensionId: data.extensionId, commandId: data.commandId, argument: data.argument)
            return
        default:
            break
        }

        // Handle regular commands
        if let result = onExecute?(UInt32(selectedIndex)) {
            switch result {
            case .success, .openSettings, .quit:
                hide()
            case .successKeepOpen, .needsInput:
                break // Keep panel open
            case .error(let message):
                print("[Nova] Execution error: \(message)")
                NSSound.beep()
            }
        }
    }

    private func executeExtensionCommand(extensionId: String, commandId: String, argument: String?) {
        guard let response = onExecuteExtension?(extensionId, commandId, argument) else {
            NSSound.beep()
            return
        }

        if let error = response.error {
            print("[Nova] Extension error: \(error)")
            NSSound.beep()
            return
        }

        if let component = response.component {
            currentExtensionId = extensionId
            showExtensionView(component: component)
        }

        if response.shouldClose {
            hide()
        }
    }

    private func showExtensionView(component: ExtensionComponent) {
        // Create navigation stack if needed
        if navigationStack == nil {
            let navStack = NavigationStack(frame: scrollView.bounds)
            navStack.translatesAutoresizingMaskIntoConstraints = false
            navStack.delegate = self
            navigationStack = navStack
        }

        guard let navStack = navigationStack, let contentView = contentView else { return }

        // Hide search results, show extension view
        scrollView.isHidden = true
        searchField.isHidden = true
        divider.isHidden = true

        if navStack.superview == nil {
            contentView.addSubview(navStack)
            NSLayoutConstraint.activate([
                navStack.topAnchor.constraint(equalTo: contentView.topAnchor, constant: 8),
                navStack.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
                navStack.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
                navStack.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -8),
            ])
        }

        navStack.isHidden = false
        navStack.push(component, title: nil)
        isShowingExtension = true
    }

    private func hideExtensionView() {
        navigationStack?.isHidden = true
        navigationStack?.clear()
        scrollView.isHidden = false
        searchField.isHidden = false
        divider.isHidden = false
        isShowingExtension = false
        currentExtensionId = nil
        makeFirstResponder(searchField)
    }

    private func performSearch() {
        let query = searchField.stringValue

        if query.isEmpty {
            results = []
        } else if let search = onSearch {
            results = search(query)
        }

        selectedIndex = 0
        resultsTableView.reloadData()

        if !results.isEmpty {
            resultsTableView.selectRowIndexes(IndexSet(integer: 0), byExtendingSelection: false)
        }
    }
}

// MARK: - NSTextFieldDelegate

extension SearchPanel: NSTextFieldDelegate {
    func controlTextDidChange(_ obj: Notification) {
        performSearch()
    }

    func control(_ control: NSControl, textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        switch commandSelector {
        case #selector(insertNewline(_:)):
            // Return key - execute selected result
            executeSelected()
            return true

        case #selector(moveUp(_:)):
            // Up arrow
            if selectedIndex > 0 {
                selectedIndex -= 1
                resultsTableView.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
                resultsTableView.scrollRowToVisible(selectedIndex)
            }
            return true

        case #selector(moveDown(_:)):
            // Down arrow
            if selectedIndex < results.count - 1 {
                selectedIndex += 1
                resultsTableView.selectRowIndexes(IndexSet(integer: selectedIndex), byExtendingSelection: false)
                resultsTableView.scrollRowToVisible(selectedIndex)
            }
            return true

        case #selector(cancelOperation(_:)):
            // Escape key
            if isShowingExtension {
                hideExtensionView()
            } else {
                hide()
            }
            return true

        default:
            return false
        }
    }
}

// MARK: - NSTableViewDataSource

extension SearchPanel: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return results.count
    }
}

// MARK: - NSTableViewDelegate

extension SearchPanel: NSTableViewDelegate {
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard row < results.count else { return nil }

        let result = results[row]

        let identifier = NSUserInterfaceItemIdentifier("ResultCell")
        var cell = tableView.makeView(withIdentifier: identifier, owner: nil) as? ResultCellView

        if cell == nil {
            cell = ResultCellView()
            cell?.identifier = identifier
        }

        cell?.configure(with: result, row: row)
        return cell
    }

    func tableViewSelectionDidChange(_ notification: Notification) {
        selectedIndex = resultsTableView.selectedRow
    }

    func tableView(_ tableView: NSTableView, shouldSelectRow row: Int) -> Bool {
        return true
    }
}

// MARK: - Result Cell View

final class ResultCellView: NSTableCellView {
    private let containerView: NSView
    private let iconView: NSImageView
    private let titleLabel: NSTextField
    private let subtitleLabel: NSTextField
    private let shortcutLabel: NSTextField

    override init(frame frameRect: NSRect) {
        containerView = NSView()
        containerView.wantsLayer = true
        containerView.layer?.cornerRadius = 8
        containerView.translatesAutoresizingMaskIntoConstraints = false

        iconView = NSImageView()
        iconView.imageScaling = .scaleProportionallyUpOrDown
        iconView.translatesAutoresizingMaskIntoConstraints = false

        titleLabel = NSTextField(labelWithString: "")
        titleLabel.font = .systemFont(ofSize: 14, weight: .medium)
        titleLabel.textColor = .labelColor
        titleLabel.lineBreakMode = .byTruncatingTail
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        subtitleLabel = NSTextField(labelWithString: "")
        subtitleLabel.font = .systemFont(ofSize: 12)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.lineBreakMode = .byTruncatingTail
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        shortcutLabel = NSTextField(labelWithString: "")
        shortcutLabel.font = .systemFont(ofSize: 11, weight: .medium)
        shortcutLabel.textColor = .tertiaryLabelColor
        shortcutLabel.alignment = .right
        shortcutLabel.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        addSubview(containerView)
        containerView.addSubview(iconView)
        containerView.addSubview(titleLabel)
        containerView.addSubview(subtitleLabel)
        containerView.addSubview(shortcutLabel)

        NSLayoutConstraint.activate([
            containerView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 8),
            containerView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -8),
            containerView.topAnchor.constraint(equalTo: topAnchor, constant: 2),
            containerView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -2),

            iconView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: 10),
            iconView.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 36),
            iconView.heightAnchor.constraint(equalToConstant: 36),

            titleLabel.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 12),
            titleLabel.trailingAnchor.constraint(equalTo: shortcutLabel.leadingAnchor, constant: -8),
            titleLabel.topAnchor.constraint(equalTo: containerView.topAnchor, constant: 8),

            subtitleLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            subtitleLabel.trailingAnchor.constraint(equalTo: titleLabel.trailingAnchor),
            subtitleLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 2),

            shortcutLabel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -10),
            shortcutLabel.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
            shortcutLabel.widthAnchor.constraint(equalToConstant: 40),
        ])
    }

    override var backgroundStyle: NSView.BackgroundStyle {
        didSet {
            // Update colors based on selection state
            if backgroundStyle == .emphasized {
                containerView.layer?.backgroundColor = NSColor.controlAccentColor.withAlphaComponent(0.2).cgColor
                titleLabel.textColor = .labelColor
                subtitleLabel.textColor = .secondaryLabelColor
            } else {
                containerView.layer?.backgroundColor = nil
                titleLabel.textColor = .labelColor
                subtitleLabel.textColor = .secondaryLabelColor
            }
        }
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configure(with result: SearchResult, row: Int) {
        titleLabel.stringValue = result.title
        subtitleLabel.stringValue = result.subtitle
        iconView.image = loadIcon(for: result)
        shortcutLabel.stringValue = row == 0 ? "↵" : ""
    }

    private func loadIcon(for result: SearchResult) -> NSImage? {
        // Try to load custom icon first
        if let iconPath = result.icon {
            if iconPath.hasPrefix("/") {
                // File path - for app icons (.icns files)
                if let image = NSImage(contentsOfFile: iconPath) {
                    return image
                }
            } else if iconPath.count <= 4 && iconPath.unicodeScalars.first?.properties.isEmoji == true {
                // Emoji - create image from text
                return emojiToImage(iconPath)
            }
        }

        // Fallback to type-specific system icons
        let symbolName: String
        switch result {
        case .app: symbolName = "app.fill"
        case .command: symbolName = "terminal.fill"
        case .alias: symbolName = "arrow.right.circle.fill"
        case .quicklink, .quicklinkWithQuery: symbolName = "link"
        case .script, .scriptWithArgument: symbolName = "applescript.fill"
        case .extensionCommand, .extensionCommandWithArg: symbolName = "puzzlepiece.extension.fill"
        case .denoCommand, .denoCommandWithArg: symbolName = "puzzlepiece.extension.fill"
        case .calculation: symbolName = "equal.circle.fill"
        case .clipboardItem: symbolName = "doc.on.clipboard"
        case .fileResult: symbolName = "doc.fill"
        case .emojiResult: return emojiToImage(result.icon ?? "")
        case .unitConversion: symbolName = "arrow.left.arrow.right"
        }

        return NSImage(systemSymbolName: symbolName, accessibilityDescription: nil)
    }

    private func emojiToImage(_ emoji: String) -> NSImage? {
        let size = NSSize(width: 32, height: 32)
        let image = NSImage(size: size)
        image.lockFocus()

        let font = NSFont.systemFont(ofSize: 24)
        let attributes: [NSAttributedString.Key: Any] = [.font: font]
        let string = NSAttributedString(string: emoji, attributes: attributes)
        let stringSize = string.size()
        let point = NSPoint(
            x: (size.width - stringSize.width) / 2,
            y: (size.height - stringSize.height) / 2
        )
        string.draw(at: point)

        image.unlockFocus()
        return image
    }
}

// MARK: - NavigationStackDelegate

extension SearchPanel: NavigationStackDelegate {
    func navigationStack(_ stack: NavigationStack, didTriggerCallback callbackId: String, payload: [String: Any]) {
        guard let extensionId = currentExtensionId else { return }

        if let response = onExtensionEvent?(extensionId, callbackId, payload) {
            if let error = response.error {
                print("[Nova] Extension callback error: \(error)")
                return
            }

            if let component = response.component {
                // Update the view with new component
                stack.replace(with: component, title: nil)
            }

            if response.shouldClose {
                hide()
            }
        }
    }

    func navigationStackDidBecomeEmpty(_ stack: NavigationStack) {
        hideExtensionView()
    }
}
