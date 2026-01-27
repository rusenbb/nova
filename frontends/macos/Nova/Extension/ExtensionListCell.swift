//
//  ExtensionListCell.swift
//  Nova
//
//  Table cell view for rendering List.Item with title, subtitle, icon, and accessories.
//  Uses theme tokens from Theme.swift for consistent styling.
//

import Cocoa

/// Table cell view for rendering a ListItem.
final class ExtensionListCell: NSTableCellView {
    private let theme = Theme.shared
    private let containerView: NSView
    private let iconView: NSImageView
    private let titleLabel: NSTextField
    private let subtitleLabel: NSTextField
    private let accessoryStackView: NSStackView

    override init(frame frameRect: NSRect) {
        let theme = Theme.shared

        containerView = NSView()
        containerView.wantsLayer = true
        containerView.layer?.cornerRadius = theme.radiusMd
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

        accessoryStackView = NSStackView()
        accessoryStackView.orientation = .horizontal
        accessoryStackView.spacing = theme.spacingSm
        accessoryStackView.alignment = .centerY
        accessoryStackView.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        addSubview(containerView)
        containerView.addSubview(iconView)
        containerView.addSubview(titleLabel)
        containerView.addSubview(subtitleLabel)
        containerView.addSubview(accessoryStackView)

        setupConstraints()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    private func setupConstraints() {
        NSLayoutConstraint.activate([
            containerView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: theme.spacingSm),
            containerView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -theme.spacingSm),
            containerView.topAnchor.constraint(equalTo: topAnchor, constant: theme.listItemSpacing),
            containerView.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -theme.listItemSpacing),

            iconView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor, constant: theme.listItemPaddingH),
            iconView.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
            iconView.widthAnchor.constraint(equalToConstant: theme.extensionIconSize),
            iconView.heightAnchor.constraint(equalToConstant: theme.extensionIconSize),

            titleLabel.leadingAnchor.constraint(equalTo: iconView.trailingAnchor, constant: theme.listItemPaddingH),
            titleLabel.trailingAnchor.constraint(lessThanOrEqualTo: accessoryStackView.leadingAnchor, constant: -theme.spacingSm),
            titleLabel.topAnchor.constraint(equalTo: containerView.topAnchor, constant: 7),

            subtitleLabel.leadingAnchor.constraint(equalTo: titleLabel.leadingAnchor),
            subtitleLabel.trailingAnchor.constraint(equalTo: titleLabel.trailingAnchor),
            subtitleLabel.topAnchor.constraint(equalTo: titleLabel.bottomAnchor, constant: 1),

            accessoryStackView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor, constant: -theme.listItemPaddingH),
            accessoryStackView.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
        ])
    }

    // MARK: - Selection Styling

    override var backgroundStyle: NSView.BackgroundStyle {
        didSet {
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

    // MARK: - Configuration

    func configure(with item: ListItem) {
        titleLabel.stringValue = item.title
        subtitleLabel.stringValue = item.subtitle ?? ""
        subtitleLabel.isHidden = item.subtitle == nil

        // Configure icon
        iconView.image = loadIcon(from: item.icon)

        // Configure accessories
        configureAccessories(item.accessories)
    }

    // MARK: - Icon Loading

    private func loadIcon(from icon: ComponentIcon?) -> NSImage? {
        guard let icon = icon else {
            // Default icon
            return NSImage(systemSymbolName: "doc.fill", accessibilityDescription: nil)
        }

        switch icon {
        case .system(let name):
            return NSImage(systemSymbolName: name, accessibilityDescription: nil)

        case .url(let urlString):
            // Load image from URL asynchronously
            if let url = URL(string: urlString) {
                loadImageAsync(from: url)
            }
            return NSImage(systemSymbolName: "photo", accessibilityDescription: nil)

        case .asset(let name):
            return NSImage(named: name)

        case .emoji(let emoji):
            return emojiToImage(emoji)

        case .text(let text, let color):
            return textToImage(text, color: color)
        }
    }

    private func loadImageAsync(from url: URL) {
        URLSession.shared.dataTask(with: url) { [weak self] data, _, _ in
            guard let data = data, let image = NSImage(data: data) else { return }
            DispatchQueue.main.async {
                self?.iconView.image = image
            }
        }.resume()
    }

    private func emojiToImage(_ emoji: String) -> NSImage {
        let size = NSSize(width: 24, height: 24)
        let image = NSImage(size: size)
        image.lockFocus()

        let font = NSFont.systemFont(ofSize: 18)
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

    private func textToImage(_ text: String, color: String?) -> NSImage {
        let size = NSSize(width: 24, height: 24)
        let image = NSImage(size: size)
        image.lockFocus()

        // Draw background circle
        let bgColor = colorFromHex(color) ?? .controlAccentColor
        bgColor.setFill()
        NSBezierPath(ovalIn: NSRect(origin: .zero, size: size)).fill()

        // Draw text
        let font = NSFont.systemFont(ofSize: 12, weight: .bold)
        let attributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor.white
        ]
        let string = NSAttributedString(string: String(text.prefix(2)), attributes: attributes)
        let stringSize = string.size()
        let point = NSPoint(
            x: (size.width - stringSize.width) / 2,
            y: (size.height - stringSize.height) / 2
        )
        string.draw(at: point)

        image.unlockFocus()
        return image
    }

    // MARK: - Accessories

    private func configureAccessories(_ accessories: [Accessory]) {
        // Clear existing accessories
        accessoryStackView.arrangedSubviews.forEach { $0.removeFromSuperview() }

        for accessory in accessories {
            let view = createAccessoryView(for: accessory)
            accessoryStackView.addArrangedSubview(view)
        }
    }

    private func createAccessoryView(for accessory: Accessory) -> NSView {
        switch accessory {
        case .text(let text):
            let label = NSTextField(labelWithString: text)
            label.font = theme.font(size: .sm)
            label.textColor = theme.foregroundTertiaryColor
            return label

        case .icon(let icon, let text):
            let stack = NSStackView()
            stack.orientation = .horizontal
            stack.spacing = theme.spacingXs

            let imageView = NSImageView()
            imageView.image = loadIcon(from: icon)
            imageView.imageScaling = .scaleProportionallyUpOrDown
            imageView.widthAnchor.constraint(equalToConstant: theme.iconSizeSm).isActive = true
            imageView.heightAnchor.constraint(equalToConstant: theme.iconSizeSm).isActive = true
            stack.addArrangedSubview(imageView)

            if let text = text {
                let label = NSTextField(labelWithString: text)
                label.font = theme.font(size: .sm)
                label.textColor = theme.foregroundTertiaryColor
                stack.addArrangedSubview(label)
            }

            return stack

        case .tag(let value, let color):
            return createTagView(value: value, color: color)

        case .date(let dateString, let format):
            let label = NSTextField(labelWithString: formatDate(dateString, format: format))
            label.font = theme.font(size: .sm)
            label.textColor = theme.foregroundTertiaryColor
            return label
        }
    }

    private func createTagView(value: String, color: String?) -> NSView {
        let container = NSView()
        container.wantsLayer = true
        container.layer?.cornerRadius = theme.radiusSm
        container.layer?.backgroundColor = (colorFromHex(color) ?? theme.accentColor).withAlphaComponent(0.2).cgColor

        let label = NSTextField(labelWithString: value)
        label.font = theme.font(size: .xs, weight: .medium)
        label.textColor = colorFromHex(color) ?? theme.accentColor
        label.translatesAutoresizingMaskIntoConstraints = false

        container.addSubview(label)

        NSLayoutConstraint.activate([
            label.leadingAnchor.constraint(equalTo: container.leadingAnchor, constant: 6),
            label.trailingAnchor.constraint(equalTo: container.trailingAnchor, constant: -6),
            label.topAnchor.constraint(equalTo: container.topAnchor, constant: 2),
            label.bottomAnchor.constraint(equalTo: container.bottomAnchor, constant: -2),
        ])

        return container
    }

    // MARK: - Date Formatting

    private func formatDate(_ dateString: String, format: DateDisplayFormat?) -> String {
        let formatter = ISO8601DateFormatter()
        guard let date = formatter.date(from: dateString) else {
            return dateString
        }

        switch format ?? .relative {
        case .relative:
            let relativeFormatter = RelativeDateTimeFormatter()
            relativeFormatter.unitsStyle = .abbreviated
            return relativeFormatter.localizedString(for: date, relativeTo: Date())

        case .absolute:
            let dateFormatter = DateFormatter()
            dateFormatter.dateStyle = .medium
            dateFormatter.timeStyle = .none
            return dateFormatter.string(from: date)

        case .time:
            let dateFormatter = DateFormatter()
            dateFormatter.dateStyle = .none
            dateFormatter.timeStyle = .short
            return dateFormatter.string(from: date)
        }
    }

    // MARK: - Color Utilities

    private func colorFromHex(_ hex: String?) -> NSColor? {
        guard let hex = hex else { return nil }

        var hexString = hex.trimmingCharacters(in: .whitespacesAndNewlines)
        if hexString.hasPrefix("#") {
            hexString.removeFirst()
        }

        guard hexString.count == 6,
              let rgb = UInt64(hexString, radix: 16) else {
            return nil
        }

        let red = CGFloat((rgb >> 16) & 0xFF) / 255.0
        let green = CGFloat((rgb >> 8) & 0xFF) / 255.0
        let blue = CGFloat(rgb & 0xFF) / 255.0

        return NSColor(red: red, green: green, blue: blue, alpha: 1.0)
    }
}
