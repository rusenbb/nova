//
//  ActionPanelBar.swift
//  Nova
//
//  Bottom bar displaying available actions with keyboard shortcuts.
//

import Cocoa

/// Bottom bar for displaying actions with keyboard shortcuts.
final class ActionPanelBar: NSView {
    private let stackView: NSStackView
    private let separatorView: NSBox
    private let moreActionsButton: NSButton
    private let moreActionsMenu: NSMenu

    private var actions: [ComponentAction] = []
    private let maxVisibleActions = 3

    /// Callback when an action is triggered.
    var onAction: ((String) -> Void)?

    // MARK: - Initialization

    override init(frame frameRect: NSRect) {
        stackView = NSStackView()
        stackView.orientation = .horizontal
        stackView.spacing = 16
        stackView.alignment = .centerY
        stackView.translatesAutoresizingMaskIntoConstraints = false

        separatorView = NSBox()
        separatorView.boxType = .separator
        separatorView.translatesAutoresizingMaskIntoConstraints = false

        moreActionsButton = NSButton()
        moreActionsButton.title = "More..."
        moreActionsButton.bezelStyle = .accessoryBarAction
        moreActionsButton.isBordered = false
        moreActionsButton.font = .systemFont(ofSize: 12)
        moreActionsButton.contentTintColor = .secondaryLabelColor
        moreActionsButton.translatesAutoresizingMaskIntoConstraints = false
        moreActionsButton.isHidden = true

        moreActionsMenu = NSMenu()

        super.init(frame: frameRect)

        setupLayout()
        moreActionsButton.target = self
        moreActionsButton.action = #selector(showMoreActions)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupLayout() {
        wantsLayer = true
        layer?.backgroundColor = NSColor.controlBackgroundColor.withAlphaComponent(0.5).cgColor

        addSubview(separatorView)
        addSubview(stackView)
        addSubview(moreActionsButton)

        NSLayoutConstraint.activate([
            separatorView.topAnchor.constraint(equalTo: topAnchor),
            separatorView.leadingAnchor.constraint(equalTo: leadingAnchor),
            separatorView.trailingAnchor.constraint(equalTo: trailingAnchor),

            stackView.topAnchor.constraint(equalTo: separatorView.bottomAnchor, constant: 6),
            stackView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 12),
            stackView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -6),

            moreActionsButton.centerYAnchor.constraint(equalTo: stackView.centerYAnchor),
            moreActionsButton.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -12),
        ])
    }

    // MARK: - Public API

    /// Configure with an action panel.
    func configure(with actionPanel: ActionPanel?) {
        // Clear existing actions
        stackView.arrangedSubviews.forEach { $0.removeFromSuperview() }
        moreActionsMenu.removeAllItems()
        actions = actionPanel?.children ?? []

        guard !actions.isEmpty else {
            isHidden = true
            return
        }

        isHidden = false

        // Add visible actions
        let visibleActions = Array(actions.prefix(maxVisibleActions))
        for (index, action) in visibleActions.enumerated() {
            let actionView = createActionView(for: action, isPrimary: index == 0)
            stackView.addArrangedSubview(actionView)
        }

        // Show "More..." button if there are additional actions
        if actions.count > maxVisibleActions {
            moreActionsButton.isHidden = false

            for action in actions.dropFirst(maxVisibleActions) {
                let menuItem = NSMenuItem(
                    title: action.title,
                    action: #selector(menuActionSelected(_:)),
                    keyEquivalent: ""
                )
                menuItem.target = self
                menuItem.representedObject = action.id

                // Add shortcut hint to menu item
                if let shortcut = action.shortcut {
                    menuItem.keyEquivalentModifierMask = modifierMask(from: shortcut.modifiers)
                    menuItem.keyEquivalent = shortcut.key
                }

                moreActionsMenu.addItem(menuItem)
            }
        } else {
            moreActionsButton.isHidden = true
        }
    }

    // MARK: - Action View Creation

    private func createActionView(for action: ComponentAction, isPrimary: Bool) -> NSView {
        let container = NSStackView()
        container.orientation = .horizontal
        container.spacing = 6
        container.alignment = .centerY

        // Action button
        let button = NSButton()
        button.title = action.title
        button.bezelStyle = isPrimary ? .rounded : .accessoryBarAction
        button.isBordered = isPrimary
        button.font = .systemFont(ofSize: 12, weight: isPrimary ? .medium : .regular)

        if action.style == .destructive {
            button.contentTintColor = .systemRed
        } else if !isPrimary {
            button.contentTintColor = .controlAccentColor
        }

        button.target = self
        button.action = #selector(actionButtonPressed(_:))
        button.tag = action.id.hashValue

        // Store action ID
        objc_setAssociatedObject(button, &AssociatedKeys.actionId, action.id, .OBJC_ASSOCIATION_RETAIN)

        container.addArrangedSubview(button)

        // Shortcut label
        if let shortcut = action.shortcut {
            let shortcutLabel = NSTextField(labelWithString: formatShortcut(shortcut))
            shortcutLabel.font = .systemFont(ofSize: 10)
            shortcutLabel.textColor = .tertiaryLabelColor
            container.addArrangedSubview(shortcutLabel)
        }

        return container
    }

    // MARK: - Shortcut Formatting

    private func formatShortcut(_ shortcut: KeyboardShortcut) -> String {
        var parts: [String] = []

        for modifier in shortcut.modifiers {
            switch modifier {
            case .cmd:
                parts.append("\u{2318}") // Command symbol
            case .ctrl:
                parts.append("\u{2303}") // Control symbol
            case .alt:
                parts.append("\u{2325}") // Option symbol
            case .shift:
                parts.append("\u{21E7}") // Shift symbol
            }
        }

        // Format key
        let keyDisplay: String
        switch shortcut.key.lowercased() {
        case "enter", "return":
            keyDisplay = "\u{21A9}" // Return symbol
        case "backspace", "delete":
            keyDisplay = "\u{232B}" // Delete symbol
        case "tab":
            keyDisplay = "\u{21E5}" // Tab symbol
        case "escape", "esc":
            keyDisplay = "\u{238B}" // Escape symbol
        case "space":
            keyDisplay = "\u{2423}" // Space symbol
        case "up":
            keyDisplay = "\u{2191}" // Up arrow
        case "down":
            keyDisplay = "\u{2193}" // Down arrow
        case "left":
            keyDisplay = "\u{2190}" // Left arrow
        case "right":
            keyDisplay = "\u{2192}" // Right arrow
        default:
            keyDisplay = shortcut.key.uppercased()
        }

        parts.append(keyDisplay)
        return parts.joined()
    }

    private func modifierMask(from modifiers: [KeyModifier]) -> NSEvent.ModifierFlags {
        var mask: NSEvent.ModifierFlags = []

        for modifier in modifiers {
            switch modifier {
            case .cmd:
                mask.insert(.command)
            case .ctrl:
                mask.insert(.control)
            case .alt:
                mask.insert(.option)
            case .shift:
                mask.insert(.shift)
            }
        }

        return mask
    }

    // MARK: - Actions

    @objc private func actionButtonPressed(_ sender: NSButton) {
        guard let actionId = objc_getAssociatedObject(sender, &AssociatedKeys.actionId) as? String else { return }
        onAction?(actionId)
    }

    @objc private func showMoreActions() {
        let location = NSPoint(x: moreActionsButton.bounds.width, y: 0)
        moreActionsMenu.popUp(positioning: nil, at: location, in: moreActionsButton)
    }

    @objc private func menuActionSelected(_ sender: NSMenuItem) {
        guard let actionId = sender.representedObject as? String else { return }
        onAction?(actionId)
    }

    // MARK: - Keyboard Shortcut Handling

    /// Check if an event matches any action shortcut and trigger it.
    /// Returns true if an action was triggered.
    func handleKeyEvent(_ event: NSEvent) -> Bool {
        let modifiers = event.modifierFlags
        let key = event.charactersIgnoringModifiers?.lowercased() ?? ""

        for action in actions {
            guard let shortcut = action.shortcut else { continue }

            // Check modifiers
            var matchesModifiers = true
            var expectedModifiers: NSEvent.ModifierFlags = []

            for modifier in shortcut.modifiers {
                switch modifier {
                case .cmd:
                    expectedModifiers.insert(.command)
                case .ctrl:
                    expectedModifiers.insert(.control)
                case .alt:
                    expectedModifiers.insert(.option)
                case .shift:
                    expectedModifiers.insert(.shift)
                }
            }

            // Check if all expected modifiers are present
            let relevantModifiers: NSEvent.ModifierFlags = [.command, .control, .option, .shift]
            let actualModifiers = modifiers.intersection(relevantModifiers)

            if actualModifiers != expectedModifiers {
                matchesModifiers = false
            }

            // Check key
            let shortcutKey = shortcut.key.lowercased()
            var matchesKey = key == shortcutKey

            // Handle special keys
            if !matchesKey {
                switch event.keyCode {
                case 36: // Return
                    matchesKey = shortcutKey == "enter" || shortcutKey == "return"
                case 51: // Backspace
                    matchesKey = shortcutKey == "backspace" || shortcutKey == "delete"
                case 48: // Tab
                    matchesKey = shortcutKey == "tab"
                case 53: // Escape
                    matchesKey = shortcutKey == "escape" || shortcutKey == "esc"
                case 49: // Space
                    matchesKey = shortcutKey == "space"
                case 126: // Up
                    matchesKey = shortcutKey == "up"
                case 125: // Down
                    matchesKey = shortcutKey == "down"
                case 123: // Left
                    matchesKey = shortcutKey == "left"
                case 124: // Right
                    matchesKey = shortcutKey == "right"
                default:
                    break
                }
            }

            if matchesModifiers && matchesKey {
                onAction?(action.id)
                return true
            }
        }

        return false
    }
}

// MARK: - Associated Keys

private struct AssociatedKeys {
    static var actionId = "actionId"
}
