//
//  ExtensionFormView.swift
//  Nova
//
//  View for rendering Form components with various input controls.
//

import Cocoa

/// View for rendering extension Form components.
final class ExtensionFormView: NSView {
    private let scrollView: NSScrollView
    private let contentStackView: NSStackView
    private let loadingIndicator: NSProgressIndicator
    private let submitButton: NSButton
    private let actionPanelBar: ActionPanelBar

    private var component: FormComponent?
    private var fieldValues: [String: Any] = [:]
    private var fieldViews: [String: NSView] = [:]
    private var validationLabels: [String: NSTextField] = [:]

    /// Callback for triggering actions.
    var onAction: ((String, [String: Any]) -> Void)?

    // MARK: - Initialization

    override init(frame frameRect: NSRect) {
        // Content stack view
        contentStackView = NSStackView()
        contentStackView.orientation = .vertical
        contentStackView.alignment = .leading
        contentStackView.spacing = 16
        contentStackView.translatesAutoresizingMaskIntoConstraints = false

        // Scroll view
        scrollView = NSScrollView()
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

        // Submit button
        submitButton = NSButton(title: "Submit", target: nil, action: nil)
        submitButton.bezelStyle = .rounded
        submitButton.keyEquivalent = "\r" // Return key
        submitButton.translatesAutoresizingMaskIntoConstraints = false

        // Action panel bar
        actionPanelBar = ActionPanelBar()
        actionPanelBar.translatesAutoresizingMaskIntoConstraints = false

        super.init(frame: frameRect)

        setupLayout()
        submitButton.target = self
        submitButton.action = #selector(submitForm)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    // MARK: - Setup

    private func setupLayout() {
        // Setup scroll view with content
        let clipView = NSClipView()
        clipView.documentView = contentStackView
        clipView.drawsBackground = false
        scrollView.contentView = clipView

        addSubview(scrollView)
        addSubview(loadingIndicator)
        addSubview(actionPanelBar)

        NSLayoutConstraint.activate([
            scrollView.topAnchor.constraint(equalTo: topAnchor, constant: 16),
            scrollView.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 16),
            scrollView.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -16),
            scrollView.bottomAnchor.constraint(equalTo: actionPanelBar.topAnchor, constant: -16),

            contentStackView.topAnchor.constraint(equalTo: clipView.topAnchor),
            contentStackView.leadingAnchor.constraint(equalTo: clipView.leadingAnchor),
            contentStackView.trailingAnchor.constraint(equalTo: clipView.trailingAnchor),
            contentStackView.widthAnchor.constraint(equalTo: scrollView.widthAnchor),

            loadingIndicator.centerXAnchor.constraint(equalTo: centerXAnchor),
            loadingIndicator.centerYAnchor.constraint(equalTo: centerYAnchor),

            actionPanelBar.leadingAnchor.constraint(equalTo: leadingAnchor),
            actionPanelBar.trailingAnchor.constraint(equalTo: trailingAnchor),
            actionPanelBar.bottomAnchor.constraint(equalTo: bottomAnchor),
            actionPanelBar.heightAnchor.constraint(equalToConstant: 36),
        ])
    }

    // MARK: - Public API

    /// Configure the view with a form component.
    func configure(with component: FormComponent) {
        self.component = component
        fieldValues = [:]
        fieldViews = [:]
        validationLabels = [:]

        // Show loading state
        if component.isLoading {
            loadingIndicator.startAnimation(nil)
            loadingIndicator.isHidden = false
            scrollView.isHidden = true
        } else {
            loadingIndicator.stopAnimation(nil)
            loadingIndicator.isHidden = true
            scrollView.isHidden = false
        }

        // Clear existing fields
        contentStackView.arrangedSubviews.forEach { $0.removeFromSuperview() }

        // Build form fields
        for field in component.children {
            let fieldView = createFieldView(for: field)
            contentStackView.addArrangedSubview(fieldView)

            // Make field view fill width
            fieldView.widthAnchor.constraint(equalTo: contentStackView.widthAnchor).isActive = true
        }

        // Add submit button
        let buttonContainer = NSView()
        buttonContainer.translatesAutoresizingMaskIntoConstraints = false
        buttonContainer.addSubview(submitButton)

        NSLayoutConstraint.activate([
            submitButton.topAnchor.constraint(equalTo: buttonContainer.topAnchor, constant: 8),
            submitButton.trailingAnchor.constraint(equalTo: buttonContainer.trailingAnchor),
            submitButton.bottomAnchor.constraint(equalTo: buttonContainer.bottomAnchor),
        ])

        contentStackView.addArrangedSubview(buttonContainer)
        buttonContainer.widthAnchor.constraint(equalTo: contentStackView.widthAnchor).isActive = true

        // Configure action panel (for additional actions besides submit)
        actionPanelBar.configure(with: nil)
    }

    // MARK: - Field Creation

    private func createFieldView(for field: FormField) -> NSView {
        let container = NSStackView()
        container.orientation = .vertical
        container.alignment = .leading
        container.spacing = 4
        container.translatesAutoresizingMaskIntoConstraints = false

        // Title label
        let titleLabel = NSTextField(labelWithString: field.title)
        titleLabel.font = .systemFont(ofSize: 13, weight: .medium)
        titleLabel.textColor = .labelColor
        container.addArrangedSubview(titleLabel)

        // Field control
        let controlView: NSView
        switch field {
        case .textField(let textField):
            controlView = createTextField(textField)
            fieldValues[textField.id] = textField.defaultValue ?? ""

        case .dropdown(let dropdown):
            controlView = createDropdown(dropdown)
            fieldValues[dropdown.id] = dropdown.defaultValue ?? dropdown.options.first?.value ?? ""

        case .checkbox(let checkbox):
            controlView = createCheckbox(checkbox)
            fieldValues[checkbox.id] = checkbox.defaultValue

        case .datePicker(let datePicker):
            controlView = createDatePicker(datePicker)
            fieldValues[datePicker.id] = datePicker.defaultValue ?? ""
        }

        container.addArrangedSubview(controlView)
        fieldViews[field.id] = controlView

        // Make control fill width
        controlView.widthAnchor.constraint(equalTo: container.widthAnchor).isActive = true

        // Validation error label (hidden by default)
        let validationLabel = NSTextField(labelWithString: "")
        validationLabel.font = .systemFont(ofSize: 11)
        validationLabel.textColor = .systemRed
        validationLabel.isHidden = true
        validationLabel.translatesAutoresizingMaskIntoConstraints = false
        container.addArrangedSubview(validationLabel)
        validationLabels[field.id] = validationLabel

        return container
    }

    private func createTextField(_ field: FormTextField) -> NSView {
        let textField: NSTextField

        switch field.fieldType {
        case .password:
            textField = NSSecureTextField()
        case .number, .text:
            textField = NSTextField()
        }

        textField.placeholderString = field.placeholder
        textField.stringValue = field.defaultValue ?? ""
        textField.font = .systemFont(ofSize: 13)
        textField.translatesAutoresizingMaskIntoConstraints = false
        textField.delegate = self
        textField.tag = field.id.hashValue

        // Store field ID for delegate callbacks
        objc_setAssociatedObject(textField, &AssociatedKeys.fieldId, field.id, .OBJC_ASSOCIATION_RETAIN)

        textField.heightAnchor.constraint(equalToConstant: 28).isActive = true

        return textField
    }

    private func createDropdown(_ field: FormDropdown) -> NSView {
        let popup = NSPopUpButton()
        popup.font = .systemFont(ofSize: 13)
        popup.translatesAutoresizingMaskIntoConstraints = false

        // Add options
        for option in field.options {
            let item = NSMenuItem(title: option.title, action: nil, keyEquivalent: "")
            item.representedObject = option.value
            popup.menu?.addItem(item)
        }

        // Select default value
        if let defaultValue = field.defaultValue,
           let index = field.options.firstIndex(where: { $0.value == defaultValue }) {
            popup.selectItem(at: index)
        }

        popup.target = self
        popup.action = #selector(dropdownChanged(_:))

        // Store field ID
        objc_setAssociatedObject(popup, &AssociatedKeys.fieldId, field.id, .OBJC_ASSOCIATION_RETAIN)

        popup.heightAnchor.constraint(equalToConstant: 28).isActive = true

        return popup
    }

    private func createCheckbox(_ field: FormCheckbox) -> NSView {
        let checkbox = NSButton(checkboxWithTitle: field.label ?? "", target: self, action: #selector(checkboxChanged(_:)))
        checkbox.state = field.defaultValue ? .on : .off
        checkbox.font = .systemFont(ofSize: 13)
        checkbox.translatesAutoresizingMaskIntoConstraints = false

        // Store field ID
        objc_setAssociatedObject(checkbox, &AssociatedKeys.fieldId, field.id, .OBJC_ASSOCIATION_RETAIN)

        return checkbox
    }

    private func createDatePicker(_ field: FormDatePicker) -> NSView {
        let datePicker = NSDatePicker()
        datePicker.datePickerStyle = .textFieldAndStepper
        datePicker.datePickerElements = field.includeTime ? [.yearMonthDay, .hourMinute] : [.yearMonthDay]
        datePicker.font = .systemFont(ofSize: 13)
        datePicker.translatesAutoresizingMaskIntoConstraints = false

        // Set default value
        if let defaultValue = field.defaultValue {
            let formatter = ISO8601DateFormatter()
            if let date = formatter.date(from: defaultValue) {
                datePicker.dateValue = date
            }
        }

        datePicker.target = self
        datePicker.action = #selector(dateChanged(_:))

        // Store field ID
        objc_setAssociatedObject(datePicker, &AssociatedKeys.fieldId, field.id, .OBJC_ASSOCIATION_RETAIN)

        datePicker.heightAnchor.constraint(equalToConstant: 28).isActive = true

        return datePicker
    }

    // MARK: - Field Change Handlers

    @objc private func dropdownChanged(_ sender: NSPopUpButton) {
        guard let fieldId = objc_getAssociatedObject(sender, &AssociatedKeys.fieldId) as? String else { return }

        if let selectedItem = sender.selectedItem,
           let value = selectedItem.representedObject as? String {
            fieldValues[fieldId] = value
            notifyChange()
        }
    }

    @objc private func checkboxChanged(_ sender: NSButton) {
        guard let fieldId = objc_getAssociatedObject(sender, &AssociatedKeys.fieldId) as? String else { return }

        fieldValues[fieldId] = sender.state == .on
        notifyChange()
    }

    @objc private func dateChanged(_ sender: NSDatePicker) {
        guard let fieldId = objc_getAssociatedObject(sender, &AssociatedKeys.fieldId) as? String else { return }

        let formatter = ISO8601DateFormatter()
        fieldValues[fieldId] = formatter.string(from: sender.dateValue)
        notifyChange()
    }

    private func notifyChange() {
        guard let callback = component?.onChange else { return }
        onAction?(callback, ["values": fieldValues])
    }

    // MARK: - Validation

    /// Validate all form fields.
    /// Returns true if all validations pass.
    private func validateForm() -> Bool {
        guard let component = component else { return false }

        var isValid = true

        for field in component.children {
            let fieldValid = validateField(field)
            if !fieldValid {
                isValid = false
            }
        }

        return isValid
    }

    private func validateField(_ field: FormField) -> Bool {
        guard case .textField(let textField) = field,
              let validation = textField.validation else {
            // No validation rules, always valid
            hideValidationError(for: field.id)
            return true
        }

        let value = fieldValues[field.id] as? String ?? ""

        // Required validation
        if validation.required && value.isEmpty {
            showValidationError(for: field.id, message: "This field is required")
            return false
        }

        // Min length validation
        if let minLength = validation.minLength, value.count < minLength {
            showValidationError(for: field.id, message: "Minimum \(minLength) characters required")
            return false
        }

        // Max length validation
        if let maxLength = validation.maxLength, value.count > maxLength {
            showValidationError(for: field.id, message: "Maximum \(maxLength) characters allowed")
            return false
        }

        // Pattern validation
        if let pattern = validation.pattern {
            do {
                let regex = try NSRegularExpression(pattern: pattern, options: [])
                let range = NSRange(value.startIndex..<value.endIndex, in: value)
                if regex.firstMatch(in: value, options: [], range: range) == nil {
                    showValidationError(for: field.id, message: "Invalid format")
                    return false
                }
            } catch {
                // Invalid regex pattern
            }
        }

        hideValidationError(for: field.id)
        return true
    }

    private func showValidationError(for fieldId: String, message: String) {
        guard let label = validationLabels[fieldId] else { return }
        label.stringValue = message
        label.isHidden = false

        // Highlight field
        if let textField = fieldViews[fieldId] as? NSTextField {
            textField.layer?.borderColor = NSColor.systemRed.cgColor
            textField.layer?.borderWidth = 1
            textField.layer?.cornerRadius = 4
        }
    }

    private func hideValidationError(for fieldId: String) {
        guard let label = validationLabels[fieldId] else { return }
        label.isHidden = true

        // Remove field highlight
        if let textField = fieldViews[fieldId] as? NSTextField {
            textField.layer?.borderWidth = 0
        }
    }

    // MARK: - Form Submission

    @objc private func submitForm() {
        guard validateForm() else {
            NSSound.beep()
            return
        }

        guard let callback = component?.onSubmit else { return }
        onAction?(callback, ["values": fieldValues])
    }

    // MARK: - Keyboard Navigation

    override var acceptsFirstResponder: Bool { true }

    override func keyDown(with event: NSEvent) {
        // Return key submits form (handled by button keyEquivalent)
        // Tab key moves between fields (handled automatically)
        super.keyDown(with: event)
    }
}

// MARK: - NSTextFieldDelegate

extension ExtensionFormView: NSTextFieldDelegate {
    func controlTextDidChange(_ obj: Notification) {
        guard let textField = obj.object as? NSTextField,
              let fieldId = objc_getAssociatedObject(textField, &AssociatedKeys.fieldId) as? String else {
            return
        }

        fieldValues[fieldId] = textField.stringValue
        notifyChange()

        // Clear validation error on edit
        hideValidationError(for: fieldId)
    }

    func controlTextDidEndEditing(_ obj: Notification) {
        guard let textField = obj.object as? NSTextField,
              let fieldId = objc_getAssociatedObject(textField, &AssociatedKeys.fieldId) as? String else {
            return
        }

        // Validate on blur
        if let component = component,
           let field = component.children.first(where: { $0.id == fieldId }) {
            _ = validateField(field)
        }
    }
}

// MARK: - Associated Keys

private struct AssociatedKeys {
    static var fieldId = "fieldId"
}
