//
//  SearchPanel.swift
//  Nova
//
//  NSPanel-based search interface that can appear over fullscreen apps.
//  Uses theme tokens from Theme.swift for consistent styling.
//

import Cocoa

final class SearchPanel: NSPanel {
    // Theme reference for styling
    private let theme = Theme.shared

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
    var onCheckPermissions: ((String, String, @escaping (Bool) -> Void) -> Void)?
    var onHide: (() -> Void)?

    // MARK: - Initialization

    override init(
        contentRect: NSRect,
        styleMask style: NSWindow.StyleMask,
        backing backingStoreType: NSWindow.BackingStoreType,
        defer flag: Bool
    ) {
        // Get theme values
        let theme = Theme.shared

        // Create search field
        searchField = NSTextField()
        searchField.placeholderString = "Search apps, commands, and more..."
        searchField.font = .systemFont(ofSize: theme.searchFieldFontSize, weight: .light)
        searchField.isBordered = false
        searchField.focusRingType = .none
        searchField.drawsBackground = false
        searchField.textColor = theme.foregroundColor
        searchField.translatesAutoresizingMaskIntoConstraints = false

        // Create table view for results
        resultsTableView = NSTableView()
        resultsTableView.headerView = nil
        resultsTableView.backgroundColor = .clear
        resultsTableView.selectionHighlightStyle = .sourceList
        resultsTableView.rowHeight = theme.listItemHeight
        resultsTableView.intercellSpacing = NSSize(width: 0, height: theme.listItemSpacing)
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
        visualEffect.layer?.cornerRadius = theme.panelCornerRadius
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
            // Search field - use theme spacing
            searchField.topAnchor.constraint(equalTo: contentView.topAnchor, constant: theme.searchFieldPaddingV),
            searchField.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: theme.searchFieldPaddingH),
            searchField.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -theme.searchFieldPaddingH),
            searchField.heightAnchor.constraint(equalToConstant: theme.searchFieldHeight - theme.searchFieldPaddingV * 2),

            // Divider - use theme spacing
            divider.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: theme.spacingMd),
            divider.leadingAnchor.constraint(equalTo: contentView.leadingAnchor, constant: theme.dividerMargin),
            divider.trailingAnchor.constraint(equalTo: contentView.trailingAnchor, constant: -theme.dividerMargin),

            // Scroll view - use theme spacing
            scrollView.topAnchor.constraint(equalTo: divider.bottomAnchor, constant: theme.spacingSm),
            scrollView.leadingAnchor.constraint(equalTo: contentView.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: contentView.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: contentView.bottomAnchor, constant: -theme.spacingSm),
        ])
    }

    private func setupDelegates() {
        searchField.delegate = self
        resultsTableView.dataSource = self
        resultsTableView.delegate = self
    }

    // MARK: - Public API

    func show() {
        // Center on screen using theme dimensions
        if let screen = NSScreen.main {
            let screenFrame = screen.frame
            let panelWidth = theme.panelWidth
            let panelHeight = theme.panelHeight

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
        // Check permissions first
        if let checkPermissions = onCheckPermissions {
            checkPermissions(extensionId, commandId) { [weak self] allowed in
                if allowed {
                    self?.performExtensionExecution(extensionId: extensionId, commandId: commandId, argument: argument)
                } else {
                    print("[Nova] Permission denied for extension: \(extensionId)")
                    NSSound.beep()
                }
            }
        } else {
            // No permission check callback - execute directly
            performExtensionExecution(extensionId: extensionId, commandId: commandId, argument: argument)
        }
    }

    private func performExtensionExecution(extensionId: String, commandId: String, argument: String?) {
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
    private let theme = Theme.shared
    private let containerView: NSView
    private let iconView: NSImageView
    private let titleLabel: NSTextField
    private let subtitleLabel: NSTextField
    private let shortcutLabel: NSTextField

    override init(frame frameRect: NSRect) {
        let theme = Theme.shared

        containerView = NSView()
        containerView.wantsLayer = true
        containerView.layer?.cornerRadius = theme.listItemCornerRadius
        containerView.translatesAutoresizingMaskIntoConstraints = false

        iconView = NSImageView()
        iconView.imageScaling = .scaleProportionallyUpOrDown
        iconView.translatesAutoresizingMaskIntoConstraints = false

        titleLabel = NSTextField(labelWithString: "")
        titleLabel.font = theme.font(size: .md, weight: .medium)
        titleLabel.textColor = theme.foregroundColor
        titleLabel.lineBreakMode = .byTruncatingTail
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        subtitleLabel = NSTextField(labelWithString: "")
        subtitleLabel.font = theme.font(size: .sm)
        subtitleLabel.textColor = theme.foregroundSecondaryColor
        subtitleLabel.lineBreakMode = .byTruncatingTail
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        shortcutLabel = NSTextField(labelWithString: "")
        shortcutLabel.font = theme.font(size: .sm, weight: .medium)
        shortcutLabel.textColor = theme.foregroundTertiaryColor
        shortcutLabel.alignment = .right
        shortcutLabel.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        addSubview(containerView)
        containerView.addSubview(iconView)
        containerView.addSubview(titleLabel)
        containerView.addSubview(subtitleLabel)
        containerView.addSubview(shortcutLabel)

        NSLayoutConstraint.activate([
            containerView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: theme.spacingSm),
            containerView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -theme.spacingSm),
            containerView.topAnchor.constraint(equalTo: topAnchor, constant: theme.listItemSpacing),
            containerView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -theme.listItemSpacing),

            iconView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: theme.listItemPaddingH),
            iconView.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: theme.listItemIconSize),
            iconView.heightAnchor.constraint(equalToConstant: theme.listItemIconSize),

            titleLabel.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: theme.spacingMd),
            titleLabel.trailingAnchor.constraint(equalTo: shortcutLabel.leadingAnchor, constant: -theme.spacingSm),
            titleLabel.topAnchor.constraint(equalTo: containerView.topAnchor, constant: theme.listItemPaddingV),

            subtitleLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            subtitleLabel.trailingAnchor.constraint(equalTo: titleLabel.trailingAnchor),
            subtitleLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: theme.radiusXs),

            shortcutLabel.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -theme.listItemPaddingH),
            shortcutLabel.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
            shortcutLabel.widthAnchor.constraint(equalToConstant: 40),
        ])
    }

    override var backgroundStyle: NSView.BackgroundStyle {
        didSet {
            // Update colors based on selection state using theme colors
            if backgroundStyle == .emphasized {
                containerView.layer?.backgroundColor = theme.selectionBackgroundColor.cgColor
                titleLabel.textColor = theme.foregroundColor
                subtitleLabel.textColor = theme.foregroundSecondaryColor
            } else {
                containerView.layer?.backgroundColor = nil
                titleLabel.textColor = theme.foregroundColor
                subtitleLabel.textColor = theme.foregroundSecondaryColor
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
