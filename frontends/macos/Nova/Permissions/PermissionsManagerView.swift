//
//  PermissionsManagerView.swift
//  Nova
//
//  Permissions management view for extension settings.
//

import Cocoa

/// Delegate for permissions management actions.
protocol PermissionsManagerDelegate: AnyObject {
    func permissionsManagerDidRevokePermission(extensionId: String, permission: String)
    func permissionsManagerDidRevokeAll(extensionId: String)
}

/// A view that displays and manages extension permissions.
final class PermissionsManagerView: NSView {
    // MARK: - Properties

    weak var delegate: PermissionsManagerDelegate?
    weak var core: NovaCore?

    private var extensions: [ExtensionPermissionEntry] = []

    // MARK: - UI Elements

    private let titleLabel = NSTextField(labelWithString: "Extension Permissions")
    private let scrollView = NSScrollView()
    private let tableView = NSTableView()
    private let emptyLabel = NSTextField(labelWithString: "No extensions have been granted permissions.")

    // MARK: - Initialization

    init(core: NovaCore) {
        self.core = core
        super.init(frame: .zero)

        setupUI()
        loadPermissions()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupUI() {
        // Title
        titleLabel.font = .systemFont(ofSize: 16, weight: .semibold)
        titleLabel.textColor = .labelColor
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        // Table view setup
        tableView.headerView = nil
        tableView.backgroundColor = .clear
        tableView.selectionHighlightStyle = .none
        tableView.rowHeight = 72
        tableView.intercellSpacing = NSSize(width: 0, height: 8)
        tableView.dataSource = self
        tableView.delegate = self

        let column = NSTableColumn(identifier: NSUserInterfaceItemIdentifier("permission"))
        column.width = 400
        tableView.addTableColumn(column)

        // Scroll view
        scrollView.documentView = tableView
        scrollView.hasVerticalScroller = true
        scrollView.hasHorizontalScroller = false
        scrollView.drawsBackground = false
        scrollView.translatesAutoresizingMaskIntoConstraints = false

        // Empty state
        emptyLabel.font = .systemFont(ofSize: 13)
        emptyLabel.textColor = .secondaryLabelColor
        emptyLabel.alignment = .center
        emptyLabel.isHidden = true
        emptyLabel.translatesAutoresizingMaskIntoConstraints = false

        addSubview(titleLabel)
        addSubview(scrollView)
        addSubview(emptyLabel)

        NSLayoutConstraint.activate([
            titleLabel.topAnchor.constraint(equalTo: topAnchor, constant: 16),
            titleLabel.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 16),

            scrollView.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 16),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: bottomAnchor),

            emptyLabel.centerXAnchor.constraint(equalTo: centerXAnchor),
            emptyLabel.centerYAnchor.constraint(equalTo: centerYAnchor),
        ])
    }

    // MARK: - Data Loading

    func loadPermissions() {
        guard let response = core?.listPermissions() else {
            extensions = []
            updateEmptyState()
            tableView.reloadData()
            return
        }

        extensions = response.extensions
        updateEmptyState()
        tableView.reloadData()
    }

    private func updateEmptyState() {
        emptyLabel.isHidden = !extensions.isEmpty
        scrollView.isHidden = extensions.isEmpty
    }
}

// MARK: - NSTableViewDataSource

extension PermissionsManagerView: NSTableViewDataSource {
    func numberOfRows(in tableView: NSTableView) -> Int {
        return extensions.count
    }
}

// MARK: - NSTableViewDelegate

extension PermissionsManagerView: NSTableViewDelegate {
    func tableView(_ tableView: NSTableView, viewFor tableColumn: NSTableColumn?, row: Int) -> NSView? {
        guard row < extensions.count else { return nil }

        let entry = extensions[row]
        let identifier = NSUserInterfaceItemIdentifier("ExtensionPermissionCell")

        var cell = tableView.makeView(withIdentifier: identifier, owner: nil) as? ExtensionPermissionCellView
        if cell == nil {
            cell = ExtensionPermissionCellView()
            cell?.identifier = identifier
        }

        cell?.configure(with: entry)
        cell?.delegate = self

        return cell
    }
}

// MARK: - ExtensionPermissionCellDelegate

extension PermissionsManagerView: ExtensionPermissionCellDelegate {
    func extensionPermissionCellDidRevokeAll(extensionId: String) {
        _ = core?.revokeAllPermissions(extensionId: extensionId)
        delegate?.permissionsManagerDidRevokeAll(extensionId: extensionId)
        loadPermissions()
    }

    func extensionPermissionCellDidRevokePermission(extensionId: String, permission: String) {
        _ = core?.revokePermission(extensionId: extensionId, permission: permission)
        delegate?.permissionsManagerDidRevokePermission(extensionId: extensionId, permission: permission)
        loadPermissions()
    }
}

// MARK: - Extension Permission Cell

protocol ExtensionPermissionCellDelegate: AnyObject {
    func extensionPermissionCellDidRevokeAll(extensionId: String)
    func extensionPermissionCellDidRevokePermission(extensionId: String, permission: String)
}

final class ExtensionPermissionCellView: NSTableCellView {
    weak var delegate: ExtensionPermissionCellDelegate?
    private var entry: ExtensionPermissionEntry?

    private let containerView = NSView()
    private let titleLabel = NSTextField(labelWithString: "")
    private let subtitleLabel = NSTextField(labelWithString: "")
    private let permissionsLabel = NSTextField(labelWithString: "")
    private let revokeButton = NSButton()

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)

        containerView.wantsLayer = true
        containerView.layer?.backgroundColor = NSColor.controlBackgroundColor.cgColor
        containerView.layer?.cornerRadius = 8
        containerView.translatesAutoresizingMaskIntoConstraints = false

        titleLabel.font = .systemFont(ofSize: 13, weight: .medium)
        titleLabel.textColor = .labelColor
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        subtitleLabel.font = .systemFont(ofSize: 11)
        subtitleLabel.textColor = .tertiaryLabelColor
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        permissionsLabel.font = .systemFont(ofSize: 11)
        permissionsLabel.textColor = .secondaryLabelColor
        permissionsLabel.translatesAutoresizingMaskIntoConstraints = false

        revokeButton.title = "Revoke All"
        revokeButton.bezelStyle = .rounded
        revokeButton.controlSize = .small
        revokeButton.target = self
        revokeButton.action = #selector(revokeAllTapped)
        revokeButton.translatesAutoresizingMaskIntoConstraints = false

        addSubview(containerView)
        containerView.addSubview(titleLabel)
        containerView.addSubview(subtitleLabel)
        containerView.addSubview(permissionsLabel)
        containerView.addSubview(revokeButton)

        NSLayoutConstraint.activate([
            containerView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 8),
            containerView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -8),
            containerView.topAnchor.constraint(equalTo: topAnchor, constant: 4),
            containerView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -4),

            titleLabel.topAnchor.constraint(equalTo: containerView.topAnchor, constant: 10),
            titleLabel.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: 12),
            titleLabel.trailingAnchor.constraint(lessThanOrEqualTo: revokeButton.leadingAnchor, constant: -8),

            subtitleLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 2),
            subtitleLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),

            permissionsLabel.topAnchor.constraint(equalTo: subtitleLabel.bottomAnchor, constant: 4),
            permissionsLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            permissionsLabel.trailingAnchor.constraint(lessThanOrEqualTo: revokeButton.leadingAnchor, constant: -8),

            revokeButton.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -12),
            revokeButton.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
        ])
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func configure(with entry: ExtensionPermissionEntry) {
        self.entry = entry

        titleLabel.stringValue = entry.displayName
        subtitleLabel.stringValue = "Last updated: \(entry.formattedDate)"
        permissionsLabel.stringValue = "Permissions: \(entry.permissions.joined(separator: ", "))"
    }

    @objc private func revokeAllTapped() {
        guard let entry = entry else { return }

        // Show confirmation
        let alert = NSAlert()
        alert.messageText = "Revoke Permissions?"
        alert.informativeText = "This will revoke all permissions for \"\(entry.displayName)\". The extension will need to request permissions again."
        alert.addButton(withTitle: "Revoke")
        alert.addButton(withTitle: "Cancel")
        alert.alertStyle = .warning

        let response = alert.runModal()
        if response == .alertFirstButtonReturn {
            delegate?.extensionPermissionCellDidRevokeAll(extensionId: entry.extensionId)
        }
    }
}

// MARK: - Permissions Manager Window Controller

final class PermissionsManagerWindowController: NSWindowController {
    private var permissionsView: PermissionsManagerView?

    init(core: NovaCore) {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 500, height: 400),
            styleMask: [.titled, .closable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "Manage Extension Permissions"
        window.minSize = NSSize(width: 400, height: 300)

        super.init(window: window)

        let view = PermissionsManagerView(core: core)
        view.translatesAutoresizingMaskIntoConstraints = false

        window.contentView = view
        permissionsView = view

        window.center()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func showWindow() {
        window?.makeKeyAndOrderFront(nil)
        permissionsView?.loadPermissions()
    }
}
