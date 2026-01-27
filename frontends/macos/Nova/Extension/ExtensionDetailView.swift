//
//  ExtensionDetailView.swift
//  Nova
//
//  View for rendering Detail components with markdown content and metadata sidebar.
//  Uses theme tokens from Theme.swift for consistent styling.
//

import Cocoa
import WebKit

/// View for rendering extension Detail components.
final class ExtensionDetailView: NSView {
    private let theme = Theme.shared
    private let splitView: NSSplitView
    private let markdownWebView: WKWebView
    private let metadataScrollView: NSScrollView
    private let metadataStackView: NSStackView
    private let loadingIndicator: NSProgressIndicator
    private let actionPanelBar: ActionPanelBar

    private var component: DetailComponent?

    /// Callback for triggering actions.
    var onAction: ((String, [String: Any]) -> Void)?

    // MARK: - Initialization

    override init(frame frameRect: NSRect) {
        // Configure WebView for markdown
        let config = WKWebViewConfiguration()
        config.preferences.setValue(true, forKey: "developerExtrasEnabled")
        markdownWebView = WKWebView(frame: .zero, configuration: config)
        markdownWebView.translatesAutoresizingMaskIntoConstraints = false

        // Metadata scroll view
        metadataScrollView = NSScrollView()
        metadataScrollView.hasVerticalScroller = true
        metadataScrollView.hasHorizontalScroller = false
        metadataScrollView.drawsBackground = false
        metadataScrollView.translatesAutoresizingMaskIntoConstraints = false

        // Metadata stack view
        metadataStackView = NSStackView()
        metadataStackView.orientation = .vertical
        metadataStackView.alignment = .leading
        metadataStackView.spacing = 12
        metadataStackView.translatesAutoresizingMaskIntoConstraints = false

        // Split view
        splitView = NSSplitView()
        splitView.isVertical = true
        splitView.dividerStyle = .thin
        splitView.translatesAutoresizingMaskIntoConstraints = false

        // Loading indicator
        loadingIndicator = NSProgressIndicator()
        loadingIndicator.style = .spinning
        loadingIndicator.controlSize = .regular
        loadingIndicator.isIndeterminate = true
        loadingIndicator.translatesAutoresizingMaskIntoConstraints = false
        loadingIndicator.isHidden = true

        // Action panel bar
        actionPanelBar = ActionPanelBar()
        actionPanelBar.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        setupLayout()
        setupWebView()
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupLayout() {
        // Add markdown web view to split view
        let markdownContainer = NSView()
        markdownContainer.translatesAutoresizingMaskIntoConstraints = false
        markdownContainer.addSubview(markdownWebView)

        NSLayoutConstraint.activate([
            markdownWebView.topAnchor.constraint(equalTo: markdownContainer.topAnchor),
            markdownWebView.leadingAnchor.constraint(equalTo: markdownContainer.leadingAnchor),
            markdownWebView.trailingAnchor.constraint(equalTo: markdownContainer.trailingAnchor),
            markdownWebView.bottomAnchor.constraint(equalTo: markdownContainer.bottomAnchor),
        ])

        // Setup metadata scroll view
        let metadataContainer = NSView()
        metadataContainer.translatesAutoresizingMaskIntoConstraints = false

        let clipView = NSClipView()
        clipView.documentView = metadataStackView
        clipView.drawsBackground = false
        metadataScrollView.contentView = clipView

        metadataContainer.addSubview(metadataScrollView)

        NSLayoutConstraint.activate([
            metadataScrollView.topAnchor.constraint(equalTo: metadataContainer.topAnchor),
            metadataScrollView.leadingAnchor.constraint(equalTo: metadataContainer.leadingAnchor),
            metadataScrollView.trailingAnchor.constraint(equalTo: metadataContainer.trailingAnchor),
            metadataScrollView.bottomAnchor.constraint(equalTo: metadataContainer.bottomAnchor),

            metadataStackView.topAnchor.constraint(equalTo: clipView.topAnchor, constant: 12),
            metadataStackView.leadingAnchor.constraint(equalTo: clipView.leadingAnchor, constant: 12),
            metadataStackView.trailingAnchor.constraint(equalTo: clipView.trailingAnchor, constant: -12),
        ])

        splitView.addArrangedSubview(markdownContainer)
        splitView.addArrangedSubview(metadataContainer)

        addSubview(splitView)
        addSubview(loadingIndicator)
        addSubview(actionPanelBar)

        NSLayoutConstraint.activate([
            splitView.topAnchor.constraint(equalTo: topAnchor),
            splitView.leadingAnchor.constraint(equalTo: leadingAnchor),
            splitView.trailingAnchor.constraint(equalTo: trailingAnchor),
            splitView.bottomAnchor.constraint(equalTo: actionPanelBar.topAnchor),

            loadingIndicator.centerXAnchor.constraint(equalTo: centerXAnchor),
            loadingIndicator.centerYAnchor.constraint(equalTo: centerYAnchor),

            actionPanelBar.leadingAnchor.constraint(equalTo: leadingAnchor),
            actionPanelBar.trailingAnchor.constraint(equalTo: trailingAnchor),
            actionPanelBar.bottomAnchor.constraint(equalTo: bottomAnchor),
            actionPanelBar.heightAnchor.constraint(equalToConstant: 36),
        ])

        // Set split view proportions
        splitView.setHoldingPriority(.defaultLow, forSubviewAt: 0)
        splitView.setHoldingPriority(.defaultHigh, forSubviewAt: 1)

        actionPanelBar.onAction = { [weak self] actionId in
            self?.executeAction(actionId)
        }
    }

    private func setupWebView() {
        // Make web view background transparent
        markdownWebView.setValue(false, forKey: "drawsBackground")
    }

    // MARK: - Public API

    /// Configure the view with a detail component.
    func configure(with component: DetailComponent) {
        self.component = component

        // Show loading state
        if component.isLoading {
            loadingIndicator.startAnimation(nil)
            loadingIndicator.isHidden = false
            splitView.isHidden = true
        } else {
            loadingIndicator.stopAnimation(nil)
            loadingIndicator.isHidden = true
            splitView.isHidden = false
        }

        // Render markdown
        if let markdown = component.markdown {
            renderMarkdown(markdown)
        } else {
            markdownWebView.loadHTMLString("<html><body></body></html>", baseURL: nil)
        }

        // Configure metadata sidebar
        configureMetadata(component.metadata)

        // Show/hide metadata sidebar
        if component.metadata == nil || component.metadata?.children.isEmpty == true {
            splitView.arrangedSubviews[1].isHidden = true
        } else {
            splitView.arrangedSubviews[1].isHidden = false
        }

        // Configure action panel
        actionPanelBar.configure(with: component.actions)
    }

    // MARK: - Markdown Rendering

    private func renderMarkdown(_ markdown: String) {
        let html = convertMarkdownToHTML(markdown)
        let styledHTML = wrapWithStyles(html)
        markdownWebView.loadHTMLString(styledHTML, baseURL: nil)
    }

    /// Simple markdown to HTML converter.
    /// For production, consider using a proper markdown library.
    private func convertMarkdownToHTML(_ markdown: String) -> String {
        var html = markdown

        // Headers
        html = html.replacingOccurrences(
            of: #"^### (.+)$"#,
            with: "<h3>$1</h3>",
            options: .regularExpression,
            range: nil
        )
        html = html.replacingOccurrences(
            of: #"^## (.+)$"#,
            with: "<h2>$1</h2>",
            options: .regularExpression,
            range: nil
        )
        html = html.replacingOccurrences(
            of: #"^# (.+)$"#,
            with: "<h1>$1</h1>",
            options: .regularExpression,
            range: nil
        )

        // Bold
        html = html.replacingOccurrences(
            of: #"\*\*(.+?)\*\*"#,
            with: "<strong>$1</strong>",
            options: .regularExpression,
            range: nil
        )

        // Italic
        html = html.replacingOccurrences(
            of: #"\*(.+?)\*"#,
            with: "<em>$1</em>",
            options: .regularExpression,
            range: nil
        )

        // Code blocks
        html = html.replacingOccurrences(
            of: #"```(\w*)\n([\s\S]*?)```"#,
            with: "<pre><code class=\"$1\">$2</code></pre>",
            options: .regularExpression,
            range: nil
        )

        // Inline code
        html = html.replacingOccurrences(
            of: #"`(.+?)`"#,
            with: "<code>$1</code>",
            options: .regularExpression,
            range: nil
        )

        // Links
        html = html.replacingOccurrences(
            of: #"\[(.+?)\]\((.+?)\)"#,
            with: "<a href=\"$2\">$1</a>",
            options: .regularExpression,
            range: nil
        )

        // Line breaks - convert double newlines to paragraphs
        let paragraphs = html.components(separatedBy: "\n\n")
        html = paragraphs
            .map { p in
                let trimmed = p.trimmingCharacters(in: .whitespacesAndNewlines)
                if trimmed.hasPrefix("<h") || trimmed.hasPrefix("<pre") || trimmed.hasPrefix("<ul") || trimmed.hasPrefix("<ol") {
                    return trimmed
                }
                return "<p>\(trimmed)</p>"
            }
            .joined(separator: "\n")

        return html
    }

    private func wrapWithStyles(_ html: String) -> String {
        // Use theme colors for consistent styling (always dark mode in Nova)
        let bgColor = theme.data.colors.background
        let textColor = theme.data.colors.foreground
        let linkColor = theme.data.colors.accent
        let codeBackground = theme.data.colors.backgroundElevated
        let borderColor = theme.data.colors.border

        return """
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="UTF-8">
            <style>
                * {
                    box-sizing: border-box;
                }
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
                    font-size: 14px;
                    line-height: 1.6;
                    color: \(textColor);
                    background-color: \(bgColor);
                    padding: 16px;
                    margin: 0;
                }
                h1, h2, h3, h4, h5, h6 {
                    margin-top: 24px;
                    margin-bottom: 16px;
                    font-weight: 600;
                    line-height: 1.25;
                }
                h1 { font-size: 1.75em; border-bottom: 1px solid \(borderColor); padding-bottom: 0.3em; }
                h2 { font-size: 1.5em; border-bottom: 1px solid \(borderColor); padding-bottom: 0.3em; }
                h3 { font-size: 1.25em; }
                p { margin-bottom: 16px; }
                a { color: \(linkColor); text-decoration: none; }
                a:hover { text-decoration: underline; }
                code {
                    font-family: 'SF Mono', Monaco, Menlo, Consolas, monospace;
                    font-size: 0.9em;
                    padding: 0.2em 0.4em;
                    background-color: \(codeBackground);
                    border-radius: 4px;
                }
                pre {
                    background-color: \(codeBackground);
                    border-radius: 6px;
                    padding: 16px;
                    overflow-x: auto;
                    margin-bottom: 16px;
                }
                pre code {
                    padding: 0;
                    background: none;
                }
                ul, ol {
                    padding-left: 2em;
                    margin-bottom: 16px;
                }
                li { margin-bottom: 4px; }
                blockquote {
                    border-left: 4px solid \(borderColor);
                    padding-left: 16px;
                    margin-left: 0;
                    color: \(theme.data.colors.foregroundSecondary);
                }
            </style>
        </head>
        <body>
            \(html)
        </body>
        </html>
        """
    }

    // MARK: - Metadata

    private func configureMetadata(_ metadata: DetailMetadata?) {
        // Clear existing metadata
        metadataStackView.arrangedSubviews.forEach { $0.removeFromSuperview() }

        guard let metadata = metadata else { return }

        // Add title
        let titleLabel = NSTextField(labelWithString: "METADATA")
        titleLabel.font = theme.font(size: .sm, weight: .semibold)
        titleLabel.textColor = theme.foregroundSecondaryColor
        metadataStackView.addArrangedSubview(titleLabel)

        // Add divider
        let divider = NSBox()
        divider.boxType = .separator
        divider.translatesAutoresizingMaskIntoConstraints = false
        metadataStackView.addArrangedSubview(divider)
        divider.widthAnchor.constraint(equalTo: metadataStackView.widthAnchor).isActive = true

        // Add metadata items
        for item in metadata.children {
            let itemView = createMetadataItemView(item)
            metadataStackView.addArrangedSubview(itemView)
        }
    }

    private func createMetadataItemView(_ item: MetadataItem) -> NSView {
        let container = NSStackView()
        container.orientation = .vertical
        container.alignment = .leading
        container.spacing = theme.spacingXs

        // Title
        let titleLabel = NSTextField(labelWithString: item.title)
        titleLabel.font = theme.font(size: .sm, weight: .medium)
        titleLabel.textColor = theme.foregroundSecondaryColor
        container.addArrangedSubview(titleLabel)

        // Value (text or link)
        if let link = item.link {
            let linkButton = NSButton()
            linkButton.title = link.text
            linkButton.bezelStyle = .inline
            linkButton.isBordered = false
            linkButton.font = theme.font(size: .md)
            linkButton.contentTintColor = theme.accentColor
            linkButton.target = self
            linkButton.action = #selector(openLink(_:))
            linkButton.tag = link.url.hashValue

            // Store URL for later retrieval
            objc_setAssociatedObject(linkButton, &AssociatedKeys.linkURL, link.url, .OBJC_ASSOCIATION_RETAIN)

            container.addArrangedSubview(linkButton)
        } else if let text = item.text {
            let valueStack = NSStackView()
            valueStack.orientation = .horizontal
            valueStack.spacing = 6

            // Icon if present
            if let icon = item.icon {
                let iconView = NSImageView()
                iconView.image = loadIcon(from: icon)
                iconView.imageScaling = .scaleProportionallyUpOrDown
                iconView.widthAnchor.constraint(equalToConstant: theme.iconSizeSm).isActive = true
                iconView.heightAnchor.constraint(equalToConstant: theme.iconSizeSm).isActive = true
                valueStack.addArrangedSubview(iconView)
            }

            let valueLabel = NSTextField(labelWithString: text)
            valueLabel.font = theme.font(size: .md)
            valueLabel.textColor = theme.foregroundColor
            valueLabel.lineBreakMode = .byTruncatingTail
            valueStack.addArrangedSubview(valueLabel)

            container.addArrangedSubview(valueStack)
        }

        return container
    }

    @objc private func openLink(_ sender: NSButton) {
        guard let urlString = objc_getAssociatedObject(sender, &AssociatedKeys.linkURL) as? String,
              let url = URL(string: urlString) else { return }
        NSWorkspace.shared.open(url)
    }

    private func loadIcon(from icon: ComponentIcon) -> NSImage? {
        switch icon {
        case .system(let name):
            return NSImage(systemSymbolName: name, accessibilityDescription: nil)
        case .emoji(let emoji):
            return emojiToImage(emoji)
        default:
            return nil
        }
    }

    private func emojiToImage(_ emoji: String) -> NSImage {
        let size = NSSize(width: 16, height: 16)
        let image = NSImage(size: size)
        image.lockFocus()
        let font = NSFont.systemFont(ofSize: 14)
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

    // MARK: - Actions

    private func executeAction(_ actionId: String) {
        guard let actions = component?.actions?.children,
              let action = actions.first(where: { $0.id == actionId }),
              let callbackId = action.onAction else { return }

        onAction?(callbackId, ["actionId": actionId])
    }

    // MARK: - Keyboard Navigation

    override var acceptsFirstResponder: Bool { true }

    override func keyDown(with event: NSEvent) {
        // Handle action shortcuts
        if handleActionShortcut(event) {
            return
        }
        super.keyDown(with: event)
    }

    private func handleActionShortcut(_ event: NSEvent) -> Bool {
        guard let actions = component?.actions?.children else { return false }

        let modifiers = event.modifierFlags
        let key = event.charactersIgnoringModifiers?.lowercased() ?? ""

        for action in actions {
            guard let shortcut = action.shortcut, let callbackId = action.onAction else { continue }

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

            if matchesModifiers && key == shortcut.key.lowercased() {
                onAction?(callbackId, ["actionId": action.id])
                return true
            }
        }

        return false
    }
}

// MARK: - Associated Keys

private struct AssociatedKeys {
    static var linkURL = "linkURL"
}
