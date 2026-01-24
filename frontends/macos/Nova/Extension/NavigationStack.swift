//
//  NavigationStack.swift
//  Nova
//
//  Manages a stack of extension views with push/pop navigation.
//

import Cocoa

/// Delegate protocol for navigation stack events.
protocol NavigationStackDelegate: AnyObject {
    /// Called when an action callback should be triggered.
    func navigationStack(_ stack: NavigationStack, didTriggerCallback callbackId: String, payload: [String: Any])
    /// Called when the navigation stack becomes empty (user navigated back to root).
    func navigationStackDidBecomeEmpty(_ stack: NavigationStack)
}

/// Entry in the navigation stack representing a view state.
struct NavigationEntry {
    let component: ExtensionComponent
    let title: String?
    let view: NSView
}

/// Manages push/pop navigation for extension views.
final class NavigationStack: NSView {
    weak var delegate: NavigationStackDelegate?

    private var stack: [NavigationEntry] = []
    private let containerView: NSView
    private let backButton: NSButton
    private let titleLabel: NSTextField
    private let headerView: NSView

    private let headerHeight: CGFloat = 36

    // MARK: - Initialization

    override init(frame frameRect: NSRect) {
        containerView = NSView()
        containerView.translatesAutoresizingMaskIntoConstraints = false

        backButton = NSButton()
        backButton.bezelStyle = .accessoryBarAction
        backButton.isBordered = false
        backButton.image = NSImage(systemSymbolName: "chevron.left", accessibilityDescription: "Back")
        backButton.contentTintColor = .controlAccentColor
        backButton.translatesAutoresizingMaskIntoConstraints = false
        backButton.isHidden = true

        titleLabel = NSTextField(labelWithString: "")
        titleLabel.font = .systemFont(ofSize: 13, weight: .semibold)
        titleLabel.textColor = .labelColor
        titleLabel.lineBreakMode = .byTruncatingTail
        titleLabel.translatesAutoresizingMaskIntoConstraints = false

        headerView = NSView()
        headerView.translatesAutoresizingMaskIntoConstraints = false
        headerView.isHidden = true

        super.init(frame: frameRect)

        setupLayout()
        backButton.target = self
        backButton.action = #selector(backButtonPressed)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupLayout() {
        addSubview(headerView)
        addSubview(containerView)

        headerView.addSubview(backButton)
        headerView.addSubview(titleLabel)

        let divider = NSBox()
        divider.boxType = .separator
        divider.translatesAutoresizingMaskIntoConstraints = false
        headerView.addSubview(divider)

        NSLayoutConstraint.activate([
            // Header view
            headerView.topAnchor.constraint(equalTo: topAnchor),
            headerView.leadingAnchor.constraint(equalTo: leadingAnchor),
            headerView.trailingAnchor.constraint(equalTo: trailingAnchor),
            headerView.heightAnchor.constraint(equalToConstant: headerHeight),

            // Back button
            backButton.leadingAnchor.constraint(equalTo: headerView.leadingAnchor, constant: 8),
            backButton.centerYAnchor.constraint(equalTo: headerView.centerYAnchor),
            backButton.widthAnchor.constraint(equalToConstant: 24),
            backButton.heightAnchor.constraint(equalToConstant: 24),

            // Title label
            titleLabel.leadingAnchor.constraint(equalTo: backButton.trailingAnchor, constant: 4),
            titleLabel.trailingAnchor.constraint(equalTo: headerView.trailingAnchor, constant: -8),
            titleLabel.centerYAnchor.constraint(equalTo: headerView.centerYAnchor),

            // Divider
            divider.leadingAnchor.constraint(equalTo: headerView.leadingAnchor),
            divider.trailingAnchor.constraint(equalTo: headerView.trailingAnchor),
            divider.bottomAnchor.constraint(equalTo: headerView.bottomAnchor),

            // Container view
            containerView.topAnchor.constraint(equalTo: headerView.bottomAnchor),
            containerView.leadingAnchor.constraint(equalTo: leadingAnchor),
            containerView.trailingAnchor.constraint(equalTo: trailingAnchor),
            containerView.bottomAnchor.constraint(equalTo: bottomAnchor),
        ])
    }

    // MARK: - Public API

    /// Push a new component onto the navigation stack.
    func push(_ component: ExtensionComponent, title: String? = nil, animated: Bool = true) {
        let view = createView(for: component)
        let entry = NavigationEntry(component: component, title: title, view: view)
        stack.append(entry)
        showView(view, animated: animated, direction: .push)
        updateHeader()
    }

    /// Pop the top view from the navigation stack.
    @discardableResult
    func pop(animated: Bool = true) -> NavigationEntry? {
        guard stack.count > 1 else {
            // At root, notify delegate
            if stack.count == 1 {
                clear(animated: animated)
            }
            return nil
        }

        let popped = stack.removeLast()
        if let current = stack.last {
            showView(current.view, animated: animated, direction: .pop)
        }
        updateHeader()
        return popped
    }

    /// Pop to the root view.
    func popToRoot(animated: Bool = true) {
        guard stack.count > 1 else { return }

        while stack.count > 1 {
            stack.removeLast()
        }

        if let root = stack.first {
            showView(root.view, animated: animated, direction: .pop)
        }
        updateHeader()
    }

    /// Replace the current view with a new component.
    func replace(with component: ExtensionComponent, title: String? = nil, animated: Bool = true) {
        guard !stack.isEmpty else {
            push(component, title: title, animated: animated)
            return
        }

        let view = createView(for: component)
        let entry = NavigationEntry(component: component, title: title, view: view)
        stack[stack.count - 1] = entry
        showView(view, animated: animated, direction: .push)
        updateHeader()
    }

    /// Clear the entire stack.
    func clear(animated: Bool = true) {
        stack.removeAll()
        containerView.subviews.forEach { $0.removeFromSuperview() }
        headerView.isHidden = true
        delegate?.navigationStackDidBecomeEmpty(self)
    }

    /// Returns true if the stack has more than one entry (can go back).
    var canGoBack: Bool {
        stack.count > 1
    }

    /// The current component being displayed.
    var currentComponent: ExtensionComponent? {
        stack.last?.component
    }

    /// Number of views in the stack.
    var depth: Int {
        stack.count
    }

    // MARK: - View Creation

    private func createView(for component: ExtensionComponent) -> NSView {
        switch component {
        case .list(let listComponent):
            let listView = ExtensionListView()
            listView.configure(with: listComponent)
            listView.onAction = { [weak self] callbackId, payload in
                guard let self = self else { return }
                self.delegate?.navigationStack(self, didTriggerCallback: callbackId, payload: payload)
            }
            return listView

        case .detail(let detailComponent):
            let detailView = ExtensionDetailView()
            detailView.configure(with: detailComponent)
            detailView.onAction = { [weak self] callbackId, payload in
                guard let self = self else { return }
                self.delegate?.navigationStack(self, didTriggerCallback: callbackId, payload: payload)
            }
            return detailView

        case .form(let formComponent):
            let formView = ExtensionFormView()
            formView.configure(with: formComponent)
            formView.onAction = { [weak self] callbackId, payload in
                guard let self = self else { return }
                self.delegate?.navigationStack(self, didTriggerCallback: callbackId, payload: payload)
            }
            return formView
        }
    }

    // MARK: - Animation

    private enum TransitionDirection {
        case push
        case pop
    }

    private func showView(_ view: NSView, animated: Bool, direction: TransitionDirection) {
        let oldViews = containerView.subviews

        view.translatesAutoresizingMaskIntoConstraints = false
        containerView.addSubview(view)

        NSLayoutConstraint.activate([
            view.topAnchor.constraint(equalTo: containerView.topAnchor),
            view.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
            view.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
            view.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),
        ])

        guard animated else {
            oldViews.forEach { $0.removeFromSuperview() }
            return
        }

        // Animate transition
        let offset: CGFloat = direction == .push ? bounds.width : -bounds.width
        view.layer?.position.x += offset
        view.alphaValue = 0

        NSAnimationContext.runAnimationGroup({ context in
            context.duration = 0.25
            context.timingFunction = CAMediaTimingFunction(name: .easeInEaseOut)

            view.animator().alphaValue = 1
            view.animator().layer?.position.x -= offset

            for oldView in oldViews {
                oldView.animator().alphaValue = 0
                oldView.animator().layer?.position.x -= offset
            }
        }, completionHandler: {
            oldViews.forEach { $0.removeFromSuperview() }
        })
    }

    // MARK: - Header Management

    private func updateHeader() {
        let showHeader = stack.count > 1
        headerView.isHidden = !showHeader
        backButton.isHidden = !showHeader

        if let current = stack.last {
            titleLabel.stringValue = current.title ?? ""
        } else {
            titleLabel.stringValue = ""
        }

        // Update container constraints based on header visibility
        containerView.constraints.forEach { constraint in
            if constraint.firstAnchor == containerView.topAnchor {
                constraint.isActive = false
            }
        }

        if showHeader {
            containerView.topAnchor.constraint(equalTo: headerView.bottomAnchor).isActive = true
        } else {
            containerView.topAnchor.constraint(equalTo: topAnchor).isActive = true
        }
    }

    // MARK: - Actions

    @objc private func backButtonPressed() {
        pop()
    }

    // MARK: - Keyboard Navigation

    override func keyDown(with event: NSEvent) {
        // Handle Escape to go back
        if event.keyCode == 53 && canGoBack {
            pop()
            return
        }

        // Handle Cmd+[ for back navigation
        if event.modifierFlags.contains(.command) && event.charactersIgnoringModifiers == "[" && canGoBack {
            pop()
            return
        }

        super.keyDown(with: event)
    }
}
