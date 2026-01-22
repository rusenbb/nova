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

    private var results: [SearchResult] = []
    private var selectedIndex: Int = 0
    private(set) var isPanelVisible: Bool = false

    var onSearch: ((String) -> [SearchResult])?
    var onExecute: ((UInt32) -> ExecutionResult)?
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
        resultsTableView.selectionHighlightStyle = .regular
        resultsTableView.rowHeight = 48
        resultsTableView.intercellSpacing = NSSize(width: 0, height: 4)
        resultsTableView.translatesAutoresizingMaskIntoConstraints = false

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("result"))
        column.width = 580
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
        let divider = NSBox()
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
        orderOut(nil)
        onHide?()
    }

    // MARK: - Keyboard Navigation

    override func keyDown(with event: NSEvent) {
        switch Int(event.keyCode) {
        case 53: // Escape
            hide()

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

        if let result = onExecute?(UInt32(selectedIndex)) {
            switch result {
            case .success, .openSettings, .quit:
                hide()
            case .successKeepOpen, .needsInput:
                break // Keep panel open
            case .error:
                NSSound.beep()
            }
        }
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
            hide()
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

        cell?.configure(with: result)
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
    private let iconView: NSImageView
    private let titleLabel: NSTextField
    private let subtitleLabel: NSTextField

    override init(frame frameRect: NSRect) {
        iconView = NSImageView()
        iconView.imageScaling = .scaleProportionallyUpOrDown
        iconView.translatesAutoresizingMaskIntoConstraints = false

        titleLabel = NSTextField(labelWithString: "")
        titleLabel.font = .systemFont(ofSize: 14, weight: .medium)
        titleLabel.textColor = .labelColor
        titleLabel.lineBreakMode = .byTruncatingTail
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        subtitleLabel = NSTextField(labelWithString: "")
        subtitleLabel.font = .systemFont(ofSize: 11)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.lineBreakMode = .byTruncatingTail
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        addSubview(iconView)
        addSubview(titleLabel)
        addSubview(subtitleLabel)

        NSLayoutConstraint.activate([
            iconView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            iconView.centerYAnchor.constraint(equalTo: centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 32),
            iconView.heightAnchor.constraint(equalToConstant: 32),

            titleLabel.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 12),
            titleLabel.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),
            titleLabel.topAnchor.constraint(equalTo: topAnchor, constant: 6),

            subtitleLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            subtitleLabel.trailingAnchor.constraint(equalTo: titleLabel.trailingAnchor),
            subtitleLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 2),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configure(with result: SearchResult) {
        titleLabel.stringValue = result.title
        subtitleLabel.stringValue = result.subtitle

        // Load icon
        if let iconPath = result.icon {
            if iconPath.hasPrefix("/") {
                // File path - for app icons
                iconView.image = NSImage(contentsOfFile: iconPath)
            } else if iconPath.count <= 2 {
                // Emoji
                iconView.image = nil
                // For emoji, we'd need a different approach
            } else {
                // System symbol or named image
                iconView.image = NSImage(systemSymbolName: "app.fill", accessibilityDescription: nil)
            }
        } else {
            iconView.image = NSImage(systemSymbolName: "magnifyingglass", accessibilityDescription: nil)
        }
    }
}
