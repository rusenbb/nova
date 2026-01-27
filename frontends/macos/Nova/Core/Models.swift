//
//  Models.swift
//  Nova
//
//  Codable models for deserializing JSON from the Rust core.
//

import Foundation

// MARK: - Search Response

struct SearchResponse: Codable {
    let results: [SearchResult]
}

// MARK: - Search Result

enum SearchResult: Codable {
    case app(AppData)
    case command(CommandData)
    case alias(AliasData)
    case quicklink(QuicklinkData)
    case quicklinkWithQuery(QuicklinkWithQueryData)
    case script(ScriptData)
    case scriptWithArgument(ScriptWithArgumentData)
    case extensionCommand(ExtensionCommandData)
    case extensionCommandWithArg(ExtensionCommandWithArgData)
    case denoCommand(DenoCommandData)
    case denoCommandWithArg(DenoCommandWithArgData)
    case calculation(CalculationData)
    case clipboardItem(ClipboardItemData)
    case fileResult(FileResultData)
    case emojiResult(EmojiResultData)
    case unitConversion(UnitConversionData)

    enum CodingKeys: String, CodingKey {
        case type
        case data
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        switch type {
        case "App":
            self = .app(try container.decode(AppData.self, forKey: .data))
        case "Command":
            self = .command(try container.decode(CommandData.self, forKey: .data))
        case "Alias":
            self = .alias(try container.decode(AliasData.self, forKey: .data))
        case "Quicklink":
            self = .quicklink(try container.decode(QuicklinkData.self, forKey: .data))
        case "QuicklinkWithQuery":
            self = .quicklinkWithQuery(try container.decode(QuicklinkWithQueryData.self, forKey: .data))
        case "Script":
            self = .script(try container.decode(ScriptData.self, forKey: .data))
        case "ScriptWithArgument":
            self = .scriptWithArgument(try container.decode(ScriptWithArgumentData.self, forKey: .data))
        case "ExtensionCommand":
            self = .extensionCommand(try container.decode(ExtensionCommandData.self, forKey: .data))
        case "ExtensionCommandWithArg":
            self = .extensionCommandWithArg(try container.decode(ExtensionCommandWithArgData.self, forKey: .data))
        case "DenoCommand":
            self = .denoCommand(try container.decode(DenoCommandData.self, forKey: .data))
        case "DenoCommandWithArg":
            self = .denoCommandWithArg(try container.decode(DenoCommandWithArgData.self, forKey: .data))
        case "Calculation":
            self = .calculation(try container.decode(CalculationData.self, forKey: .data))
        case "ClipboardItem":
            self = .clipboardItem(try container.decode(ClipboardItemData.self, forKey: .data))
        case "FileResult":
            self = .fileResult(try container.decode(FileResultData.self, forKey: .data))
        case "EmojiResult":
            self = .emojiResult(try container.decode(EmojiResultData.self, forKey: .data))
        case "UnitConversion":
            self = .unitConversion(try container.decode(UnitConversionData.self, forKey: .data))
        default:
            throw DecodingError.dataCorruptedError(forKey: .type, in: container, debugDescription: "Unknown type: \(type)")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .app(let data):
            try container.encode("App", forKey: .type)
            try container.encode(data, forKey: .data)
        case .command(let data):
            try container.encode("Command", forKey: .type)
            try container.encode(data, forKey: .data)
        case .alias(let data):
            try container.encode("Alias", forKey: .type)
            try container.encode(data, forKey: .data)
        case .quicklink(let data):
            try container.encode("Quicklink", forKey: .type)
            try container.encode(data, forKey: .data)
        case .quicklinkWithQuery(let data):
            try container.encode("QuicklinkWithQuery", forKey: .type)
            try container.encode(data, forKey: .data)
        case .script(let data):
            try container.encode("Script", forKey: .type)
            try container.encode(data, forKey: .data)
        case .scriptWithArgument(let data):
            try container.encode("ScriptWithArgument", forKey: .type)
            try container.encode(data, forKey: .data)
        case .extensionCommand(let data):
            try container.encode("ExtensionCommand", forKey: .type)
            try container.encode(data, forKey: .data)
        case .extensionCommandWithArg(let data):
            try container.encode("ExtensionCommandWithArg", forKey: .type)
            try container.encode(data, forKey: .data)
        case .denoCommand(let data):
            try container.encode("DenoCommand", forKey: .type)
            try container.encode(data, forKey: .data)
        case .denoCommandWithArg(let data):
            try container.encode("DenoCommandWithArg", forKey: .type)
            try container.encode(data, forKey: .data)
        case .calculation(let data):
            try container.encode("Calculation", forKey: .type)
            try container.encode(data, forKey: .data)
        case .clipboardItem(let data):
            try container.encode("ClipboardItem", forKey: .type)
            try container.encode(data, forKey: .data)
        case .fileResult(let data):
            try container.encode("FileResult", forKey: .type)
            try container.encode(data, forKey: .data)
        case .emojiResult(let data):
            try container.encode("EmojiResult", forKey: .type)
            try container.encode(data, forKey: .data)
        case .unitConversion(let data):
            try container.encode("UnitConversion", forKey: .type)
            try container.encode(data, forKey: .data)
        }
    }

    // MARK: - Display Properties

    var title: String {
        switch self {
        case .app(let data): return data.name
        case .command(let data): return data.name
        case .alias(let data): return data.name
        case .quicklink(let data): return data.name
        case .quicklinkWithQuery(let data): return data.name
        case .script(let data): return data.name
        case .scriptWithArgument(let data): return data.name
        case .extensionCommand(let data): return data.command.name
        case .extensionCommandWithArg(let data): return data.command.name
        case .denoCommand(let data): return data.title
        case .denoCommandWithArg(let data): return data.title
        case .calculation(let data): return data.result
        case .clipboardItem(let data): return data.preview
        case .fileResult(let data): return data.name
        case .emojiResult(let data): return "\(data.emoji) \(data.name)"
        case .unitConversion(let data): return data.result
        }
    }

    var subtitle: String {
        switch self {
        case .app(let data): return data.description ?? "Application"
        case .command(let data): return data.description
        case .alias(let data): return data.target
        case .quicklink(let data): return data.url
        case .quicklinkWithQuery(let data): return data.resolvedUrl
        case .script(let data): return data.description
        case .scriptWithArgument(let data): return data.description
        case .extensionCommand(let data): return data.command.description
        case .extensionCommandWithArg(let data): return data.command.description
        case .denoCommand(let data): return data.subtitle ?? "Extension"
        case .denoCommandWithArg(let data): return data.extensionId
        case .calculation(let data): return data.expression
        case .clipboardItem(_): return "Clipboard"
        case .fileResult(let data): return data.path
        case .emojiResult(let data): return data.category
        case .unitConversion(let data): return data.expression
        }
    }

    var icon: String? {
        switch self {
        case .app(let data): return data.icon
        case .command(_): return nil
        case .alias(let data): return data.icon
        case .quicklink(let data): return data.icon
        case .quicklinkWithQuery(let data): return data.icon
        case .script(let data): return data.icon
        case .scriptWithArgument(let data): return data.icon
        case .extensionCommand(_): return nil
        case .extensionCommandWithArg(_): return nil
        case .denoCommand(let data): return data.icon
        case .denoCommandWithArg(_): return nil
        case .calculation(_): return nil
        case .clipboardItem(_): return nil
        case .fileResult(_): return nil
        case .emojiResult(let data): return data.emoji
        case .unitConversion(_): return nil
        }
    }
}

// MARK: - Data Types

struct AppData: Codable {
    let id: String
    let name: String
    let exec: String
    let icon: String?
    let description: String?
    let keywords: [String]
}

struct CommandData: Codable {
    let id: String
    let name: String
    let description: String
    let icon: String?
}

struct AliasData: Codable {
    let keyword: String
    let name: String
    let target: String
    let icon: String?
}

struct QuicklinkData: Codable {
    let keyword: String
    let name: String
    let url: String
    let icon: String?
    let hasQuery: Bool

    enum CodingKeys: String, CodingKey {
        case keyword, name, url, icon
        case hasQuery = "has_query"
    }
}

struct QuicklinkWithQueryData: Codable {
    let keyword: String
    let name: String
    let resolvedUrl: String
    let icon: String?

    enum CodingKeys: String, CodingKey {
        case keyword, name, icon
        case resolvedUrl = "resolved_url"
    }
}

struct ScriptData: Codable {
    let id: String
    let name: String
    let description: String
    let icon: String?
    let path: String
    let hasArgument: Bool
    let outputMode: String

    enum CodingKeys: String, CodingKey {
        case id, name, description, icon, path
        case hasArgument = "has_argument"
        case outputMode = "output_mode"
    }
}

struct ScriptWithArgumentData: Codable {
    let id: String
    let name: String
    let description: String
    let icon: String?
    let path: String
    let argument: String
    let outputMode: String

    enum CodingKeys: String, CodingKey {
        case id, name, description, icon, path, argument
        case outputMode = "output_mode"
    }
}

struct LoadedCommandData: Codable {
    let id: String
    let extensionId: String
    let name: String
    let description: String
    let keyword: String
    let scriptPath: String
    let hasArgument: Bool
    let output: String
    let iconPath: String?

    enum CodingKeys: String, CodingKey {
        case id, name, description, keyword, output
        case extensionId = "extension_id"
        case scriptPath = "script_path"
        case hasArgument = "has_argument"
        case iconPath = "icon_path"
    }
}

struct ExtensionCommandData: Codable {
    let command: LoadedCommandData
}

struct ExtensionCommandWithArgData: Codable {
    let command: LoadedCommandData
    let argument: String
}

struct DenoCommandData: Codable {
    let extensionId: String
    let commandId: String
    let title: String
    let subtitle: String?
    let icon: String?
    let keywords: [String]

    enum CodingKeys: String, CodingKey {
        case extensionId = "extension_id"
        case commandId = "command_id"
        case title, subtitle, icon, keywords
    }
}

struct DenoCommandWithArgData: Codable {
    let extensionId: String
    let commandId: String
    let title: String
    let argument: String

    enum CodingKeys: String, CodingKey {
        case extensionId = "extension_id"
        case commandId = "command_id"
        case title, argument
    }
}

struct CalculationData: Codable {
    let expression: String
    let result: String
}

struct ClipboardItemData: Codable {
    let content: String
    let preview: String
    let timestamp: UInt64
}

struct FileResultData: Codable {
    let name: String
    let path: String
    let isDirectory: Bool

    enum CodingKeys: String, CodingKey {
        case name, path
        case isDirectory = "is_directory"
    }
}

struct EmojiResultData: Codable {
    let emoji: String
    let name: String
    let category: String
}

struct UnitConversionData: Codable {
    let expression: String
    let result: String
}

// MARK: - Execution Response

/// Rust serializes ExecutionResult with `#[serde(tag = "result", content = "message")]`
/// This produces: {"result": "Success"} or {"result": "Error", "message": "..."}
struct ExecuteResponse: Codable {
    let result: String
    let message: String?

    var executionResult: ExecutionResult {
        switch result {
        case "Success": return .success
        case "SuccessKeepOpen": return .successKeepOpen
        case "OpenSettings": return .openSettings
        case "Quit": return .quit
        case "Error": return .error(message ?? "Unknown error")
        case "NeedsInput": return .needsInput
        default: return .error("Unknown result: \(result)")
        }
    }
}

enum ExecutionResult {
    case success
    case successKeepOpen
    case openSettings
    case quit
    case error(String)
    case needsInput
}
