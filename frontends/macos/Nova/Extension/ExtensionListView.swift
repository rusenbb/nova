//
//  ExtensionListView.swift
//  Nova
//
//  NSTableView-based view for rendering List components.
//

import Cocoa

/// Row data for the table view, handling both items and section headers.
enum ListRowData {
    case sectionHeader(title: String, subtitle: String?)
    case item(ListItem, sectionIndex: Int?)
}

/// View for rendering extension List components with search filtering.
final class ExtensionListView: NSView {
    private let searchField: NSTextField
    private let tableView: NSTableView
    private let scrollView: NSScrollView
    private let loadingIndicator: NSProgressIndicator
    private let emptyLabel: NSTextField
    private let actionPanelBar: ActionPanelBar

    private var component: ListComponent?
    private var allRows: [ListRowData] = []
    private var filteredRows: [ListRowData] = []
    private var selectedIndex: Int = 0

    /// Callback for triggering actions.
    var onAction: ((String, [String: Any]) -> Void)?

    // MARK: - Initialization

    override init(frame frameRect: NSRect) {
        // Search field
        searchField = NSTextField()
        searchField.placeholderString = "Filter..."
        searchField.font = .systemFont(ofSize: 14)
        searchField.isBordered = true
        searchField.bezelStyle = .roundedBezel
        searchField.focusRingType = .none
        searchField.translatesAutoresizingMaskIntoConstraints = false

        // Table view
        tableView = NSTableView()
        tableView.headerView = nil
        tableView.backgroundColor = .clear
        tableView.selectionHighlightStyle = .sourceList
        tableView.rowHeight = 48
        tableView.intercellSpacing = NSSize(width: 0, height: 1)
        tableView.translatesAutoresizingMaskIntoConstraints = false
        tableView.style = .plain
        tableView.usesAlternatingRowBackgroundColors = false

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("content"))
        column.width = 600
        tableView.addTableColumn(column)

        // Scroll view
        scrollView = NSScrollView()
        scrollView.documentView = tableView
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.drawsBackground = false
        scrollView.translatesAutoresizingMaskIntoConstraints = false

        // Loading indicator
        loadingIndicator = NSProgressIndicator()
        loadingIndicator.style = .spinning
        loadingIndicator.controlSize = .regular
        loadingIndicator.isIndeterminate = true
        loadingIndicator.translatesAutoresizingMaskIntoConstraints = false
        loadingIndicator.isHidden = true

        // Empty label
        emptyLabel = NSTextField(labelWithString: "No items")
        emptyLabel.font = .systemFont(ofSize: 14)
        emptyLabel.textColor = .secondaryLabelColor
        emptyLabel.alignment = .center
        emptyLabel.translatesAutoresizingMaskIntoConstraints = false
        emptyLabel.isHidden = true

        // Action panel bar
        actionPanelBar = ActionPanelBar()
        actionPanelBar.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        setupLayout()
        setupDelegates()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupLayout() {
        addSubview(searchField)
        addSubview(scrollView)
        addSubview(loadingIndicator)
        addSubview(emptyLabel)
        addSubview(actionPanelBar)

        NSLayoutConstraint.activate([
            // Search field
            searchField.topAnchor.constraint(equalTo: topAnchor, constant: 8),
            searchField.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            searchField.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),
            searchField.heightAnchor.constraint(equalToConstant: 28),

            // Scroll view
            scrollView.topAnchor.constraint(equalTo: searchField.bottomAnchor, constant: 8),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: actionPanelBar.topAnchor),

            // Loading indicator
            loadingIndicator.centerXAnchor.constraint(equalTo: centerXAnchor),
            loadingIndicator.centerYAnchor.constraint(equalTo: centerYAnchor),

            // Empty label
            emptyLabel.centerXAnchor.constraint(equalTo: centerXAnchor),
            emptyLabel.centerYAnchor.constraint(equalTo: centerYAnchor),

            // Action panel bar
            actionPanelBar.leadingAnchor.constraint(equalTo: leadingAnchor),
            actionPanelBar.trailingAnchor.constraint(equalTo: trailingAnchor),
            actionPanelBar.bottomAnchor.constraint(equalTo: bottomAnchor),
            actionPanelBar.heightAnchor.constraint(equalToConstant: 36),
        ])
    }

    private func setupDelegates() {
        searchField.delegate = self
        tableView.dataSource = self
        tableView.delegate = self

        actionPanelBar.onAction = { [weak self] actionId in
            self?.executeAction(actionId)
        }
    }

    // MARK: - Public API

    /// Configure the view with a list component.
    func configure(with component: ListComponent) {
        self.component = component

        // Update search field placeholder
        searchField.placeholderString = component.searchBarPlaceholder ?? "Filter..."

        // Hide search field if filtering is disabled
        searchField.isHidden = component.filtering == .none

        // Show loading state
        if component.isLoading {
            loadingIndicator.startAnimation(nil)
            loadingIndicator.isHidden = false
            scrollView.isHidden = true
            emptyLabel.isHidden = true
        } else {
            loadingIndicator.stopAnimation(nil)
            loadingIndicator.isHidden = true
            scrollView.isHidden = false
        }

        // Build row data
        buildRowData()
        applyFilter()

        // Select first item
        selectFirstItem()

        // Update action panel for selected item
        updateActionPanel()
    }

    // MARK: - Row Data Building

    private func buildRowData() {
        allRows = []

        guard let component = component else { return }

        for child in component.children {
            switch child {
            case .item(let item):
                allRows.append(.item(item, sectionIndex: nil))

            case .section(let section):
                // Add section header if it has a title
                if let title = section.title {
                    allRows.append(.sectionHeader(title: title, subtitle: section.subtitle))
                }
                // Add all items in the section
                for item in section.children {
                    allRows.append(.item(item, sectionIndex: allRows.count))
                }
            }
        }
    }

    // MARK: - Filtering

    private func applyFilter() {
        let query = searchField.stringValue.lowercased()

        guard !query.isEmpty, component?.filtering != .none else {
            filteredRows = allRows
            reloadTable()
            return
        }

        // If custom filtering, notify delegate
        if component?.filtering == .custom, let callback = component?.onSearchChange {
            onAction?(callback, ["query": query])
            return
        }

        // Default filtering - filter by title, subtitle, and keywords
        filteredRows = allRows.filter { row in
            switch row {
            case .sectionHeader:
                // Keep section headers if they have matching items
                return true
            case .item(let item, _):
                if item.title.lowercased().contains(query) {
                    return true
                }
                if let subtitle = item.subtitle, subtitle.lowercased().contains(query) {
                    return true
                }
                if item.keywords.contains(where: { $0.lowercased().contains(query) }) {
                    return true
                }
                return false
            }
        }

        // Remove section headers that have no items after them
        filteredRows = filteredRows.filter { row in
            switch row {
            case .sectionHeader:
                // Check if there's at least one item after this header
                if let index = filteredRows.firstIndex(where: { $0.isSameAs(row) }) {
                    for i in (index + 1)..<filteredRows.count {
                        switch filteredRows[i] {
                        case .sectionHeader:
                            return false // Next row is another header, no items
                        case .item:
                            return true // Found an item
                        }
                    }
                    return false // No more rows
                }
                return false
            case .item:
                return true
            }
        }

        reloadTable()
    }

    private func reloadTable() {
        tableView.reloadData()

        let hasItems = filteredRows.contains { if case .item = $0 { return true } else { return false } }
        emptyLabel.isHidden = hasItems || component?.isLoading == true
        scrollView.isHidden = !hasItems && component?.isLoading != true
    }

    private func selectFirstItem() {
        // Find first actual item (not section header)
        for (index, row) in filteredRows.enumerated() {
            if case .item = row {
                selectedIndex = index
                tableView.selectRowIndexes(IndexSet(integer: index), byExtendingSelection: false)
                tableView.scrollRowToVisible(index)
                return
            }
        }
        selectedIndex = 0
    }

    // MARK: - Action Panel

    private func updateActionPanel() {
        guard selectedIndex < filteredRows.count else {
            actionPanelBar.configure(with: nil)
            return
        }

        switch filteredRows[selectedIndex] {
        case .item(let item, _):
            actionPanelBar.configure(with: item.actions)
        case .sectionHeader:
            actionPanelBar.configure(with: nil)
        }
    }

    private func executeAction(_ actionId: String) {
        guard selectedIndex < filteredRows.count else { return }

        switch filteredRows[selectedIndex] {
        case .item(let item, _):
            if let action = item.actions?.children.first(where: { $0.id == actionId }),
               let callbackId = action.onAction {
                onAction?(callbackId, ["itemId": item.id, "actionId": actionId])
            }
        case .sectionHeader:
            break
        }
    }

    // MARK: - Selection

    private func selectedItem() -> ListItem? {
        guard selectedIndex < filteredRows.count else { return nil }
        if case .item(let item, _) = filteredRows[selectedIndex] {
            return item
        }
        return nil
    }

    // MARK: - Keyboard Navigation

    override var acceptsFirstResponder: Bool { true }

    override func keyDown(with event: NSEvent) {
        switch Int(event.keyCode) {
        case 125: // Down arrow
            moveSelection(by: 1)

        case 126: // Up arrow
            moveSelection(by: -1)

        case 36: // Return - execute primary action
            executePrimaryAction()

        default:
            // Check for action shortcuts
            if handleActionShortcut(event) {
                return
            }
            super.keyDown(with: event)
        }
    }

    private func moveSelection(by delta: Int) {
        var newIndex = selectedIndex + delta

        // Skip section headers
        while newIndex >= 0 && newIndex < filteredRows.count {
            if case .item = filteredRows[newIndex] {
                break
            }
            newIndex += delta
        }

        guard newIndex >= 0 && newIndex < filteredRows.count else { return }

        selectedIndex = newIndex
        tableView.selectRowIndexes(IndexSet(integer: newIndex), byExtendingSelection: false)
        tableView.scrollRowToVisible(newIndex)
        updateActionPanel()

        // Notify selection change callback
        if let callback = component?.onSelectionChange, let item = selectedItem() {
            onAction?(callback, ["itemId": item.id])
        }
    }

    private func executePrimaryAction() {
        guard let item = selectedItem(),
              let firstAction = item.actions?.children.first,
              let callbackId = firstAction.onAction else { return }

        onAction?(callbackId, ["itemId": item.id, "actionId": firstAction.id])
    }

    private func handleActionShortcut(_ event: NSEvent) -> Bool {
        guard let item = selectedItem(),
              let actions = item.actions?.children else { return false }

        let modifiers = event.modifierFlags
        let key = event.charactersIgnoringModifiers?.lowercased() ?? ""

        for action in actions {
            guard let shortcut = action.shortcut, let callbackId = action.onAction else { continue }

            // Check modifiers
            var matchesModifiers = true
            for modifier in shortcut.modifiers {
                switch modifier {
                case .cmd:
                    if !modifiers.contains(.command) { matchesModifiers = false }
                case .ctrl:
                    if !modifiers.contains(.control) { matchesModifiers = false }
                case .alt:
                    if !modifiers.contains(.option) { matchesModifiers = false }
                case .shift:
                    if !modifiers.contains(.shift) { matchesModifiers = false }
                }
            }

            // Check key
            if matchesModifiers && key == shortcut.key.lowercased() {
                onAction?(callbackId, ["itemId": item.id, "actionId": action.id])
                return true
            }
        }

        return false
    }
}

// MARK: - NSTextFieldDelegate

extension ExtensionListView: NSTextFieldDelegate {
    func controlTextDidChange(_ obj: Notification) {
        applyFilter()
        selectFirstItem()
        updateActionPanel()
    }
}

// MARK: - NSTableViewDataSource

extension ExtensionListView: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return filteredRows.count
    }
}

// MARK: - NSTableViewDelegate

extension ExtensionListView: NSTableViewDelegate {
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard row < filteredRows.count else { return nil }

        switch filteredRows[row] {
        case .sectionHeader(let title, let subtitle):
            return createSectionHeaderView(title: title, subtitle: subtitle, tableView: tableView)

        case .item(let item, _):
            return createItemCellView(item: item, tableView: tableView)
        }
    }

    func tableView(_ tableView: NSTableView, heightOfRow row: Int) -> CGFloat {
        guard row < filteredRows.count else { return 48 }

        switch filteredRows[row] {
        case .sectionHeader:
            return 28
        case .item:
            return 48
        }
    }

    func tableView(_ tableView: NSTableView, shouldSelectRow row: Int) -> Bool {
        guard row < filteredRows.count else { return false }
        // Don't allow selecting section headers
        if case .sectionHeader = filteredRows[row] {
            return false
        }
        return true
    }

    func tableViewSelectionDidChange(_ notification: Notification) {
        let newIndex = tableView.selectedRow
        guard newIndex >= 0 && newIndex < filteredRows.count else { return }

        selectedIndex = newIndex
        updateActionPanel()

        // Notify selection change callback
        if let callback = component?.onSelectionChange, let item = selectedItem() {
            onAction?(callback, ["itemId": item.id])
        }
    }

    // MARK: - Cell Creation

    private func createSectionHeaderView(title: String, subtitle: String?, tableView: NSTableView) -> NSView {
        let identifier = NSUserInterfaceItemIdentifier("SectionHeader")
        var view = tableView.makeView(withIdentifier: identifier, owner: nil) as? SectionHeaderView

        if view == nil {
            view = SectionHeaderView()
            view?.identifier = identifier
        }

        view?.configure(title: title, subtitle: subtitle)
        return view!
    }

    private func createItemCellView(item: ListItem, tableView: NSTableView) -> NSView {
        let identifier = NSUserInterfaceItemIdentifier("ListItemCell")
        var cell = tableView.makeView(withIdentifier: identifier, owner: nil) as? ExtensionListCell

        if cell == nil {
            cell = ExtensionListCell()
            cell?.identifier = identifier
        }

        cell?.configure(with: item)
        return cell!
    }
}

// MARK: - ListRowData Extension

extension ListRowData {
    func isSameAs(_ other: ListRowData) -> Bool {
        switch (self, other) {
        case (.sectionHeader(let t1, _), .sectionHeader(let t2, _)):
            return t1 == t2
        case (.item(let i1, _), .item(let i2, _)):
            return i1.id == i2.id
        default:
            return false
        }
    }
}

// MARK: - Section Header View

final class SectionHeaderView: NSTableCellView {
    private let titleLabel: NSTextField
    private let subtitleLabel: NSTextField

    override init(frame frameRect: NSRect) {
        titleLabel = NSTextField(labelWithString: "")
        titleLabel.font = .systemFont(ofSize: 11, weight: .semibold)
        titleLabel.textColor = .secondaryLabelColor
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        subtitleLabel = NSTextField(labelWithString: "")
        subtitleLabel.font = .systemFont(ofSize: 11)
        subtitleLabel.textColor = .tertiaryLabelColor
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        addSubview(titleLabel)
        addSubview(subtitleLabel)

        NSLayoutConstraint.activate([
            titleLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            titleLabel.centerYAnchor.constraint(equalTo: centerYAnchor),

            subtitleLabel.leadingAnchor.constraint(equalTo: titleLabel.trailingAnchor, constant: 8),
            subtitleLabel.trailingAnchor.constraint(lessThanOrEqualTo: trailingAnchor, constant: -12),
            subtitleLabel.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configure(title: String, subtitle: String?) {
        titleLabel.stringValue = title.uppercased()
        subtitleLabel.stringValue = subtitle ?? ""
        subtitleLabel.isHidden = subtitle == nil
    }
}
