//
//  ExtensionModels.swift
//  Nova
//
//  Codable models for extension component rendering.
//  These mirror the Rust component types from nova::extensions::components.
//

import Foundation

// MARK: - Root Component

/// Root component type that can be rendered by an extension.
enum ExtensionComponent: Codable {
    case list(ListComponent)
    case detail(DetailComponent)
    case form(FormComponent)

    enum CodingKeys: String, CodingKey {
        case type
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "List":
            self = .list(try ListComponent(from: decoder))
        case "Detail":
            self = .detail(try DetailComponent(from: decoder))
        case "Form":
            self = .form(try FormComponent(from: decoder))
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown component type: \(type)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .list(let component):
            try container.encode("List", forKey: .type)
            try component.encode(to: encoder)
        case .detail(let component):
            try container.encode("Detail", forKey: .type)
            try component.encode(to: encoder)
        case .form(let component):
            try container.encode("Form", forKey: .type)
            try component.encode(to: encoder)
        }
    }
}

// MARK: - List Component

/// List component - displays a searchable list of items.
struct ListComponent: Codable {
    let isLoading: Bool
    let searchBarPlaceholder: String?
    let filtering: ListFiltering
    let onSearchChange: String?
    let onSelectionChange: String?
    let children: [ListChild]

    init(
        isLoading: Bool = false,
        searchBarPlaceholder: String? = nil,
        filtering: ListFiltering = .default,
        onSearchChange: String? = nil,
        onSelectionChange: String? = nil,
        children: [ListChild] = []
    ) {
        self.isLoading = isLoading
        self.searchBarPlaceholder = searchBarPlaceholder
        self.filtering = filtering
        self.onSearchChange = onSearchChange
        self.onSelectionChange = onSelectionChange
        self.children = children
    }
}

/// Filtering behavior for the list.
enum ListFiltering: String, Codable {
    case `default` = "default"
    case none = "none"
    case custom = "custom"
}

/// A child element of a List (either an Item or a Section).
enum ListChild: Codable {
    case item(ListItem)
    case section(ListSection)

    enum CodingKeys: String, CodingKey {
        case type
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "List.Item":
            self = .item(try ListItem(from: decoder))
        case "List.Section":
            self = .section(try ListSection(from: decoder))
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown list child type: \(type)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .item(let item):
            try container.encode("List.Item", forKey: .type)
            try item.encode(to: encoder)
        case .section(let section):
            try container.encode("List.Section", forKey: .type)
            try section.encode(to: encoder)
        }
    }
}

/// A single item in a list.
struct ListItem: Codable {
    let id: String
    let title: String
    let subtitle: String?
    let icon: ComponentIcon?
    let accessories: [Accessory]
    let keywords: [String]
    let actions: ActionPanel?

    init(
        id: String,
        title: String,
        subtitle: String? = nil,
        icon: ComponentIcon? = nil,
        accessories: [Accessory] = [],
        keywords: [String] = [],
        actions: ActionPanel? = nil
    ) {
        self.id = id
        self.title = title
        self.subtitle = subtitle
        self.icon = icon
        self.accessories = accessories
        self.keywords = keywords
        self.actions = actions
    }
}

/// A section that groups list items.
struct ListSection: Codable {
    let title: String?
    let subtitle: String?
    let children: [ListItem]

    init(title: String? = nil, subtitle: String? = nil, children: [ListItem] = []) {
        self.title = title
        self.subtitle = subtitle
        self.children = children
    }
}

// MARK: - Detail Component

/// Detail component - displays markdown content with metadata.
struct DetailComponent: Codable {
    let markdown: String?
    let isLoading: Bool
    let actions: ActionPanel?
    let metadata: DetailMetadata?

    init(
        markdown: String? = nil,
        isLoading: Bool = false,
        actions: ActionPanel? = nil,
        metadata: DetailMetadata? = nil
    ) {
        self.markdown = markdown
        self.isLoading = isLoading
        self.actions = actions
        self.metadata = metadata
    }
}

/// Metadata sidebar for Detail component.
struct DetailMetadata: Codable {
    let children: [MetadataItem]

    init(children: [MetadataItem] = []) {
        self.children = children
    }
}

/// A single metadata item (key-value pair).
struct MetadataItem: Codable {
    let title: String
    let text: String?
    let icon: ComponentIcon?
    let link: MetadataLink?
}

/// A clickable link in metadata.
struct MetadataLink: Codable {
    let text: String
    let url: String
}

// MARK: - Form Component

/// Form component - collects user input.
struct FormComponent: Codable {
    let isLoading: Bool
    let onSubmit: String?
    let onChange: String?
    let children: [FormField]

    init(
        isLoading: Bool = false,
        onSubmit: String? = nil,
        onChange: String? = nil,
        children: [FormField] = []
    ) {
        self.isLoading = isLoading
        self.onSubmit = onSubmit
        self.onChange = onChange
        self.children = children
    }
}

/// A form field (text, dropdown, checkbox, or date picker).
enum FormField: Codable {
    case textField(FormTextField)
    case dropdown(FormDropdown)
    case checkbox(FormCheckbox)
    case datePicker(FormDatePicker)

    enum CodingKeys: String, CodingKey {
        case type
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "Form.TextField":
            self = .textField(try FormTextField(from: decoder))
        case "Form.Dropdown":
            self = .dropdown(try FormDropdown(from: decoder))
        case "Form.Checkbox":
            self = .checkbox(try FormCheckbox(from: decoder))
        case "Form.DatePicker":
            self = .datePicker(try FormDatePicker(from: decoder))
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown form field type: \(type)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .textField(let field):
            try container.encode("Form.TextField", forKey: .type)
            try field.encode(to: encoder)
        case .dropdown(let field):
            try container.encode("Form.Dropdown", forKey: .type)
            try field.encode(to: encoder)
        case .checkbox(let field):
            try container.encode("Form.Checkbox", forKey: .type)
            try field.encode(to: encoder)
        case .datePicker(let field):
            try container.encode("Form.DatePicker", forKey: .type)
            try field.encode(to: encoder)
        }
    }

    var id: String {
        switch self {
        case .textField(let f): return f.id
        case .dropdown(let f): return f.id
        case .checkbox(let f): return f.id
        case .datePicker(let f): return f.id
        }
    }

    var title: String {
        switch self {
        case .textField(let f): return f.title
        case .dropdown(let f): return f.title
        case .checkbox(let f): return f.title
        case .datePicker(let f): return f.title
        }
    }
}

/// Text input field.
struct FormTextField: Codable {
    let id: String
    let title: String
    let placeholder: String?
    let defaultValue: String?
    let fieldType: TextFieldType
    let validation: FieldValidation?

    init(
        id: String,
        title: String,
        placeholder: String? = nil,
        defaultValue: String? = nil,
        fieldType: TextFieldType = .text,
        validation: FieldValidation? = nil
    ) {
        self.id = id
        self.title = title
        self.placeholder = placeholder
        self.defaultValue = defaultValue
        self.fieldType = fieldType
        self.validation = validation
    }
}

/// Text field input type.
enum TextFieldType: String, Codable {
    case text
    case password
    case number
}

/// Validation rules for form fields.
struct FieldValidation: Codable {
    let required: Bool
    let pattern: String?
    let minLength: Int?
    let maxLength: Int?

    init(required: Bool = false, pattern: String? = nil, minLength: Int? = nil, maxLength: Int? = nil) {
        self.required = required
        self.pattern = pattern
        self.minLength = minLength
        self.maxLength = maxLength
    }
}

/// Dropdown/select field.
struct FormDropdown: Codable {
    let id: String
    let title: String
    let defaultValue: String?
    let options: [DropdownOption]

    init(id: String, title: String, defaultValue: String? = nil, options: [DropdownOption] = []) {
        self.id = id
        self.title = title
        self.defaultValue = defaultValue
        self.options = options
    }
}

/// An option in a dropdown.
struct DropdownOption: Codable {
    let value: String
    let title: String
    let icon: ComponentIcon?
}

/// Checkbox field.
struct FormCheckbox: Codable {
    let id: String
    let title: String
    let label: String?
    let defaultValue: Bool

    init(id: String, title: String, label: String? = nil, defaultValue: Bool = false) {
        self.id = id
        self.title = title
        self.label = label
        self.defaultValue = defaultValue
    }
}

/// Date picker field.
struct FormDatePicker: Codable {
    let id: String
    let title: String
    let defaultValue: String?
    let includeTime: Bool

    init(id: String, title: String, defaultValue: String? = nil, includeTime: Bool = false) {
        self.id = id
        self.title = title
        self.defaultValue = defaultValue
        self.includeTime = includeTime
    }
}

// MARK: - Action Components

/// Container for actions associated with a component.
struct ActionPanel: Codable {
    let title: String?
    let children: [ComponentAction]

    init(title: String? = nil, children: [ComponentAction] = []) {
        self.title = title
        self.children = children
    }
}

/// A single action that can be triggered by the user.
struct ComponentAction: Codable {
    let id: String
    let title: String
    let icon: ComponentIcon?
    let shortcut: KeyboardShortcut?
    let style: ActionStyle
    let onAction: String?

    init(
        id: String,
        title: String,
        icon: ComponentIcon? = nil,
        shortcut: KeyboardShortcut? = nil,
        style: ActionStyle = .default,
        onAction: String? = nil
    ) {
        self.id = id
        self.title = title
        self.icon = icon
        self.shortcut = shortcut
        self.style = style
        self.onAction = onAction
    }
}

/// Visual style for an action.
enum ActionStyle: String, Codable {
    case `default` = "default"
    case destructive = "destructive"
}

// MARK: - Common Types

/// Icon reference - can be a system icon, URL, asset, emoji, or text.
enum ComponentIcon: Codable {
    case system(name: String)
    case url(url: String)
    case asset(name: String)
    case emoji(emoji: String)
    case text(text: String, color: String?)

    enum CodingKeys: String, CodingKey {
        case type, name, url, emoji, text, color
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "system":
            let name = try container.decode(String.self, forKey: .name)
            self = .system(name: name)
        case "url":
            let url = try container.decode(String.self, forKey: .url)
            self = .url(url: url)
        case "asset":
            let name = try container.decode(String.self, forKey: .name)
            self = .asset(name: name)
        case "emoji":
            let emoji = try container.decode(String.self, forKey: .emoji)
            self = .emoji(emoji: emoji)
        case "text":
            let text = try container.decode(String.self, forKey: .text)
            let color = try container.decodeIfPresent(String.self, forKey: .color)
            self = .text(text: text, color: color)
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown icon type: \(type)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .system(let name):
            try container.encode("system", forKey: .type)
            try container.encode(name, forKey: .name)
        case .url(let url):
            try container.encode("url", forKey: .type)
            try container.encode(url, forKey: .url)
        case .asset(let name):
            try container.encode("asset", forKey: .type)
            try container.encode(name, forKey: .name)
        case .emoji(let emoji):
            try container.encode("emoji", forKey: .type)
            try container.encode(emoji, forKey: .emoji)
        case .text(let text, let color):
            try container.encode("text", forKey: .type)
            try container.encode(text, forKey: .text)
            try container.encodeIfPresent(color, forKey: .color)
        }
    }
}

/// Accessory displayed on the right side of list items.
enum Accessory: Codable {
    case text(text: String)
    case icon(icon: ComponentIcon, text: String?)
    case tag(value: String, color: String?)
    case date(date: String, format: DateDisplayFormat?)

    enum CodingKeys: String, CodingKey {
        case type, text, icon, value, color, date, format
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "text":
            let text = try container.decode(String.self, forKey: .text)
            self = .text(text: text)
        case "icon":
            let icon = try container.decode(ComponentIcon.self, forKey: .icon)
            let text = try container.decodeIfPresent(String.self, forKey: .text)
            self = .icon(icon: icon, text: text)
        case "tag":
            let value = try container.decode(String.self, forKey: .value)
            let color = try container.decodeIfPresent(String.self, forKey: .color)
            self = .tag(value: value, color: color)
        case "date":
            let date = try container.decode(String.self, forKey: .date)
            let format = try container.decodeIfPresent(DateDisplayFormat.self, forKey: .format)
            self = .date(date: date, format: format)
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .type,
                in: container,
                debugDescription: "Unknown accessory type: \(type)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .text(let text):
            try container.encode("text", forKey: .type)
            try container.encode(text, forKey: .text)
        case .icon(let icon, let text):
            try container.encode("icon", forKey: .type)
            try container.encode(icon, forKey: .icon)
            try container.encodeIfPresent(text, forKey: .text)
        case .tag(let value, let color):
            try container.encode("tag", forKey: .type)
            try container.encode(value, forKey: .value)
            try container.encodeIfPresent(color, forKey: .color)
        case .date(let date, let format):
            try container.encode("date", forKey: .type)
            try container.encode(date, forKey: .date)
            try container.encodeIfPresent(format, forKey: .format)
        }
    }
}

/// Date display format.
enum DateDisplayFormat: String, Codable {
    case relative
    case absolute
    case time
}

/// Keyboard shortcut definition.
struct KeyboardShortcut: Codable {
    let modifiers: [KeyModifier]
    let key: String
}

/// Keyboard modifier keys.
enum KeyModifier: String, Codable {
    case cmd
    case ctrl
    case alt
    case shift
}

// MARK: - Extension Event

/// Event sent from the frontend to an extension.
struct ExtensionEvent: Codable {
    let extensionId: String
    let callbackId: String
    let payload: [String: AnyCodable]

    init(extensionId: String, callbackId: String, payload: [String: AnyCodable] = [:]) {
        self.extensionId = extensionId
        self.callbackId = callbackId
        self.payload = payload
    }
}

/// Type-erased Codable value for dynamic payloads.
struct AnyCodable: Codable {
    let value: Any

    init(_ value: Any) {
        self.value = value
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        if container.decodeNil() {
            value = NSNull()
        } else if let bool = try? container.decode(Bool.self) {
            value = bool
        } else if let int = try? container.decode(Int.self) {
            value = int
        } else if let double = try? container.decode(Double.self) {
            value = double
        } else if let string = try? container.decode(String.self) {
            value = string
        } else if let array = try? container.decode([AnyCodable].self) {
            value = array.map { $0.value }
        } else if let dictionary = try? container.decode([String: AnyCodable].self) {
            value = dictionary.mapValues { $0.value }
        } else {
            throw DecodingError.dataCorruptedError(in: container, debugDescription: "Unable to decode value")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch value {
        case is NSNull:
            try container.encodeNil()
        case let bool as Bool:
            try container.encode(bool)
        case let int as Int:
            try container.encode(int)
        case let double as Double:
            try container.encode(double)
        case let string as String:
            try container.encode(string)
        case let array as [Any]:
            try container.encode(array.map { AnyCodable($0) })
        case let dictionary as [String: Any]:
            try container.encode(dictionary.mapValues { AnyCodable($0) })
        default:
            throw EncodingError.invalidValue(value, EncodingError.Context(
                codingPath: encoder.codingPath,
                debugDescription: "Unable to encode value of type \(type(of: value))"
            ))
        }
    }
}

// MARK: - Extension Response

/// Response from the Rust core when executing an extension or sending an event.
struct ExtensionResponse: Codable {
    let component: ExtensionComponent?
    let error: String?
    let shouldClose: Bool

    init(component: ExtensionComponent? = nil, error: String? = nil, shouldClose: Bool = false) {
        self.component = component
        self.error = error
        self.shouldClose = shouldClose
    }
}
