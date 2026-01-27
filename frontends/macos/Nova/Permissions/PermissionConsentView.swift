//
//  PermissionConsentView.swift
//  Nova
//
//  Permission consent dialog shown when an extension requests permissions.
//

import Cocoa

/// Delegate protocol for permission consent actions.
protocol PermissionConsentDelegate: AnyObject {
    func permissionConsentDidAllow(extensionId: String, permissions: [String])
    func permissionConsentDidDeny(extensionId: String)
    func permissionConsentDidAllowAll(extensionId: String)
}

/// A sheet/dialog that requests user consent for extension permissions.
final class PermissionConsentView: NSView {
    // MARK: - Properties

    weak var delegate: PermissionConsentDelegate?

    private let extensionId: String
    private let extensionTitle: String
    private let permissions: [PermissionInfo]

    private var selectedPermissions: Set<String> = []

    // MARK: - UI Elements

    private let iconView = NSImageView()
    private let titleLabel = NSTextField(labelWithString: "")
    private let subtitleLabel = NSTextField(labelWithString: "")
    private let permissionsStack = NSStackView()
    private let denyButton = NSButton()
    private let allowButton = NSButton()

    // MARK: - Initialization

    init(
        extensionId: String,
        extensionTitle: String,
        permissions: [PermissionInfo]
    ) {
        self.extensionId = extensionId
        self.extensionTitle = extensionTitle
        self.permissions = permissions

        // Select all permissions by default
        self.selectedPermissions = Set(permissions.map { $0.name })

        super.init(frame: .zero)

        setupUI()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupUI() {
        wantsLayer = true
        layer?.backgroundColor = NSColor.windowBackgroundColor.cgColor

        // Icon
        iconView.image = NSImage(systemSymbolName: "lock.shield", accessibilityDescription: "Permission")
        iconView.symbolConfiguration = .init(pointSize: 48, weight: .regular)
        iconView.contentTintColor = .controlAccentColor
        iconView.translatesAutoresizingMaskIntoConstraints = false

        // Title
        titleLabel.stringValue = "\"\(extensionTitle)\" would like to access:"
        titleLabel.font = .systemFont(ofSize: 16, weight: .semibold)
        titleLabel.textColor = .labelColor
        titleLabel.alignment = .center
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        // Subtitle
        subtitleLabel.stringValue = "Review the permissions this extension is requesting."
        subtitleLabel.font = .systemFont(ofSize: 13)
        subtitleLabel.textColor = .secondaryLabelColor
        subtitleLabel.alignment = .center
        subtitleLabel.translatesAutoresizingMaskIntoConstraints = false

        // Permissions list
        permissionsStack.orientation = .vertical
        permissionsStack.alignment = .leading
        permissionsStack.spacing = 12
        permissionsStack.translatesAutoresizingMaskIntoConstraints = false

        for permission in permissions {
            let row = createPermissionRow(permission)
            permissionsStack.addArrangedSubview(row)
        }

        // Buttons
        denyButton.title = "Don't Allow"
        denyButton.bezelStyle = .rounded
        denyButton.target = self
        denyButton.action = #selector(denyTapped)
        denyButton.translatesAutoresizingMaskIntoConstraints = false

        allowButton.title = "Allow"
        allowButton.bezelStyle = .rounded
        allowButton.keyEquivalent = "\r"
        allowButton.target = self
        allowButton.action = #selector(allowTapped)
        allowButton.translatesAutoresizingMaskIntoConstraints = false

        // Button stack
        let buttonStack = NSStackView(views: [denyButton, allowButton])
        buttonStack.orientation = .horizontal
        buttonStack.spacing = 12
        buttonStack.translatesAutoresizingMaskIntoConstraints = false

        // Main stack
        let mainStack = NSStackView(views: [iconView, titleLabel, subtitleLabel, permissionsStack, buttonStack])
        mainStack.orientation = .vertical
        mainStack.alignment = .centerX
        mainStack.spacing = 16
        mainStack.translatesAutoresizingMaskIntoConstraints = false

        mainStack.setCustomSpacing(8, after: titleLabel)
        mainStack.setCustomSpacing(24, after: subtitleLabel)
        mainStack.setCustomSpacing(24, after: permissionsStack)

        addSubview(mainStack)

        NSLayoutConstraint.activate([
            mainStack.topAnchor.constraint(equalTo: topAnchor, constant: 24),
            mainStack.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 24),
            mainStack.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -24),
            mainStack.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -24),

            permissionsStack.leadingAnchor.constraint(equalTo: mainStack.leadingAnchor),
            permissionsStack.trailingAnchor.constraint(equalTo: mainStack.trailingAnchor),
        ])
    }

    private func createPermissionRow(_ permission: PermissionInfo) -> NSView {
        let container = NSView()
        container.translatesAutoresizingMaskIntoConstraints = false

        // Checkbox
        let checkbox = NSButton(checkboxWithTitle: "", target: self, action: #selector(permissionToggled(_:)))
        checkbox.state = .on
        checkbox.tag = permissions.firstIndex(where: { $0.name == permission.name }) ?? 0
        checkbox.translatesAutoresizingMaskIntoConstraints = false

        // Icon
        let iconView = NSImageView()
        iconView.image = NSImage(systemSymbolName: permission.icon, accessibilityDescription: permission.name)
        iconView.symbolConfiguration = .init(pointSize: 20, weight: .regular)
        iconView.contentTintColor = .secondaryLabelColor
        iconView.translatesAutoresizingMaskIntoConstraints = false

        // Title and description stack
        let textStack = NSStackView()
        textStack.orientation = .vertical
        textStack.alignment = .leading
        textStack.spacing = 2
        textStack.translatesAutoresizingMaskIntoConstraints = false

        let titleLabel = NSTextField(labelWithString: permission.name.capitalized)
        titleLabel.font = .systemFont(ofSize: 13, weight: .medium)
        titleLabel.textColor = .labelColor

        let descLabel = NSTextField(labelWithString: permission.description)
        descLabel.font = .systemFont(ofSize: 11)
        descLabel.textColor = .secondaryLabelColor
        descLabel.preferredMaxLayoutWidth = 250

        textStack.addArrangedSubview(titleLabel)
        textStack.addArrangedSubview(descLabel)

        // Details label (if present)
        if let details = permission.details {
            let detailsLabel = NSTextField(labelWithString: details)
            detailsLabel.font = .systemFont(ofSize: 10)
            detailsLabel.textColor = .tertiaryLabelColor
            detailsLabel.preferredMaxLayoutWidth = 250
            textStack.addArrangedSubview(detailsLabel)
        }

        container.addSubview(checkbox)
        container.addSubview(iconView)
        container.addSubview(textStack)

        NSLayoutConstraint.activate([
            container.heightAnchor.constraint(greaterThanOrEqualToConstant: 44),

            checkbox.leadingAnchor.constraint(equalTo: container.leadingAnchor),
            checkbox.centerYAnchor.constraint(equalTo: container.centerYAnchor),

            iconView.leadingAnchor.constraint(equalTo: checkbox.trailingAnchor, constant: 8),
            iconView.centerYAnchor.constraint(equalTo: container.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: 24),
            iconView.heightAnchor.constraint(equalToConstant: 24),

            textStack.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: 8),
            textStack.trailingAnchor.constraint(equalTo: container.trailingAnchor),
            textStack.topAnchor.constraint(equalTo: container.topAnchor, constant: 4),
            textStack.bottomAnchor.constraint(equalTo: container.bottomAnchor, constant: -4),
        ])

        return container
    }

    // MARK: - Actions

    @objc private func permissionToggled(_ sender: NSButton) {
        let permission = permissions[sender.tag]
        if sender.state == .on {
            selectedPermissions.insert(permission.name)
        } else {
            selectedPermissions.remove(permission.name)
        }

        // Disable allow button if no permissions selected
        allowButton.isEnabled = !selectedPermissions.isEmpty
    }

    @objc private func denyTapped() {
        delegate?.permissionConsentDidDeny(extensionId: extensionId)
    }

    @objc private func allowTapped() {
        if selectedPermissions.count == permissions.count {
            delegate?.permissionConsentDidAllowAll(extensionId: extensionId)
        } else {
            delegate?.permissionConsentDidAllow(extensionId: extensionId, permissions: Array(selectedPermissions))
        }
    }
}

// MARK: - Permission Consent Window Controller

final class PermissionConsentWindowController: NSWindowController {
    private let extensionId: String
    private let extensionTitle: String
    private let permissions: [PermissionInfo]
    private var completion: ((Bool) -> Void)?

    private weak var core: NovaCore?

    init(
        core: NovaCore,
        extensionId: String,
        extensionTitle: String,
        permissions: [PermissionInfo]
    ) {
        self.core = core
        self.extensionId = extensionId
        self.extensionTitle = extensionTitle
        self.permissions = permissions

        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 380, height: 0),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "Permission Request"
        window.isMovableByWindowBackground = true

        super.init(window: window)

        let contentView = PermissionConsentView(
            extensionId: extensionId,
            extensionTitle: extensionTitle,
            permissions: permissions
        )
        contentView.delegate = self

        window.contentView = contentView

        // Size to fit content
        let fittingSize = contentView.fittingSize
        window.setContentSize(NSSize(width: max(380, fittingSize.width), height: fittingSize.height))
        window.center()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    func showModal(completion: @escaping (Bool) -> Void) {
        self.completion = completion

        guard let window = window else {
            completion(false)
            return
        }

        window.center()
        NSApp.runModal(for: window)
    }

    private func dismiss(allowed: Bool) {
        NSApp.stopModal()
        window?.close()
        completion?(allowed)
    }
}

// MARK: - PermissionConsentDelegate

extension PermissionConsentWindowController: PermissionConsentDelegate {
    func permissionConsentDidAllow(extensionId: String, permissions: [String]) {
        // Grant individual permissions
        for permission in permissions {
            _ = core?.grantPermission(extensionId: extensionId, permission: permission)
        }
        dismiss(allowed: true)
    }

    func permissionConsentDidDeny(extensionId: String) {
        dismiss(allowed: false)
    }

    func permissionConsentDidAllowAll(extensionId: String) {
        // Grant all permissions at once
        _ = core?.grantAllPermissions(extensionId: extensionId)
        dismiss(allowed: true)
    }
}

// MARK: - Helper Extension

extension NovaCore {
    /// Show permission consent dialog if needed, then execute completion.
    ///
    /// Returns true if execution should proceed, false if denied.
    func showPermissionConsentIfNeeded(
        extensionId: String,
        extensionTitle: String,
        completion: @escaping (Bool) -> Void
    ) {
        guard let response = checkPermissions(extensionId: extensionId) else {
            // Error checking permissions - allow for now
            completion(true)
            return
        }

        if response.needsConsent.isEmpty {
            // No consent needed
            completion(true)
            return
        }

        // Show consent dialog
        let controller = PermissionConsentWindowController(
            core: self,
            extensionId: extensionId,
            extensionTitle: extensionTitle,
            permissions: response.needsConsent
        )

        controller.showModal(completion: completion)
    }
}
