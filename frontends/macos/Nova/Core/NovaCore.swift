//
//  NovaCore.swift
//  Nova
//
//  Swift wrapper around the Rust core via C FFI.
//

import Foundation

final class NovaCore {
    private var handle: OpaquePointer?
    private let decoder = JSONDecoder()

    init?() {
        handle = nova_core_new()
        if handle == nil {
            return nil
        }
    }

    deinit {
        if let handle = handle {
            nova_core_free(handle)
        }
    }

    // MARK: - Search

    func search(query: String, maxResults: UInt32 = 10) -> [SearchResult] {
        guard let handle = handle else { return [] }

        guard let resultPtr = nova_core_search(handle, query, maxResults) else {
            return []
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return []
        }

        do {
            let response = try decoder.decode(SearchResponse.self, from: jsonData)
            return response.results
        } catch {
            print("[Nova] Failed to decode search results: \(error)")
            return []
        }
    }

    // MARK: - Execute

    func execute(index: UInt32) -> ExecutionResult {
        guard let handle = handle else { return .error }

        guard let resultPtr = nova_core_execute(handle, index) else {
            return .error
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return .error
        }

        do {
            let response = try decoder.decode(ExecuteResponse.self, from: jsonData)
            return response.result
        } catch {
            print("[Nova] Failed to decode execution result: \(error)")
            return .error
        }
    }

    // MARK: - Clipboard Polling

    func pollClipboard() {
        guard let handle = handle else { return }
        nova_core_poll_clipboard(handle)
    }

    // MARK: - Reload

    func reload() {
        guard let handle = handle else { return }
        nova_core_reload(handle)
    }

    // MARK: - Result Count

    var resultCount: UInt32 {
        guard let handle = handle else { return 0 }
        return nova_core_result_count(handle)
    }

    // MARK: - Extension Execution

    /// Execute an extension command and return the rendered component.
    func executeExtension(extensionId: String, commandId: String, argument: String? = nil) -> ExtensionResponse? {
        guard let handle = handle else { return nil }

        let resultPtr: UnsafeMutablePointer<CChar>?
        if let arg = argument {
            resultPtr = nova_core_execute_extension(handle, extensionId, commandId, arg)
        } else {
            resultPtr = nova_core_execute_extension(handle, extensionId, commandId, nil)
        }

        guard let resultPtr = resultPtr else {
            return nil
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return nil
        }

        do {
            let response = try decoder.decode(ExtensionExecuteResponseInternal.self, from: jsonData)
            return ExtensionResponse(
                component: response.component,
                error: response.error,
                shouldClose: response.shouldClose
            )
        } catch {
            print("[Nova] Failed to decode extension response: \(error)")
            return nil
        }
    }

    /// Send an event to an extension callback.
    func sendEvent(extensionId: String, callbackId: String, payload: [String: Any]) -> ExtensionResponse? {
        guard let handle = handle else { return nil }

        // Serialize payload to JSON
        guard let payloadData = try? JSONSerialization.data(withJSONObject: payload),
              let payloadString = String(data: payloadData, encoding: .utf8) else {
            return nil
        }

        guard let resultPtr = nova_core_send_event(handle, extensionId, callbackId, payloadString) else {
            return nil
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return nil
        }

        do {
            let response = try decoder.decode(ExtensionExecuteResponseInternal.self, from: jsonData)
            return ExtensionResponse(
                component: response.component,
                error: response.error,
                shouldClose: response.shouldClose
            )
        } catch {
            print("[Nova] Failed to decode event response: \(error)")
            return nil
        }
    }
}

// MARK: - Internal Response Type

/// Internal response type for decoding FFI responses.
private struct ExtensionExecuteResponseInternal: Codable {
    let success: Bool
    let error: String?
    let component: ExtensionComponent?
    let shouldClose: Bool
}
