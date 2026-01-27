//
//  NovaCore.swift
//  Nova
//
//  Swift wrapper around the Rust core via C FFI.
//
//  Thread Safety: This class is marked @MainActor to ensure all FFI calls
//  execute on the main thread. The underlying Rust code is not thread-safe
//  and expects single-threaded access to the NovaCore handle.
//

import Foundation

@MainActor
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
        guard let handle = handle else { return .error("No handle") }

        guard let resultPtr = nova_core_execute(handle, index) else {
            return .error("Execute returned null")
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return .error("Invalid UTF-8")
        }

        do {
            let response = try decoder.decode(ExecuteResponse.self, from: jsonData)
            return response.executionResult
        } catch {
            print("[Nova] Failed to decode execution result: \(error)")
            print("[Nova] JSON was: \(jsonString)")
            return .error("Decode error: \(error)")
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

// MARK: - Permission Management

extension NovaCore {
    /// Get the title of an extension by ID.
    func getExtensionTitle(extensionId: String) -> String? {
        guard let handle = handle else { return nil }

        guard let resultPtr = nova_core_get_extension_title(handle, extensionId) else {
            return nil
        }

        defer { nova_string_free(resultPtr) }
        return String(cString: resultPtr)
    }

    /// Check if an extension needs permission consent.
    func checkPermissions(extensionId: String) -> PermissionQueryResponse? {
        guard let handle = handle else { return nil }

        guard let resultPtr = nova_core_check_permissions(handle, extensionId) else {
            return nil
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return nil
        }

        do {
            return try decoder.decode(PermissionQueryResponse.self, from: jsonData)
        } catch {
            print("[Nova] Failed to decode permission response: \(error)")
            return nil
        }
    }

    /// Grant a single permission to an extension.
    func grantPermission(extensionId: String, permission: String) -> Bool {
        guard let handle = handle else { return false }
        return nova_core_grant_permission(handle, extensionId, permission) == 1
    }

    /// Grant all requested permissions to an extension.
    func grantAllPermissions(extensionId: String) -> Bool {
        guard let handle = handle else { return false }
        return nova_core_grant_all_permissions(handle, extensionId) == 1
    }

    /// Revoke a single permission from an extension.
    func revokePermission(extensionId: String, permission: String) -> Bool {
        guard let handle = handle else { return false }
        return nova_core_revoke_permission(handle, extensionId, permission) == 1
    }

    /// Revoke all permissions from an extension.
    func revokeAllPermissions(extensionId: String) -> Bool {
        guard let handle = handle else { return false }
        return nova_core_revoke_all_permissions(handle, extensionId) == 1
    }

    /// List all extensions with their permissions.
    func listPermissions() -> ExtensionPermissionsResponse? {
        guard let handle = handle else { return nil }

        guard let resultPtr = nova_core_list_permissions(handle) else {
            return nil
        }

        defer { nova_string_free(resultPtr) }

        let jsonString = String(cString: resultPtr)
        guard let jsonData = jsonString.data(using: .utf8) else {
            return nil
        }

        do {
            return try decoder.decode(ExtensionPermissionsResponse.self, from: jsonData)
        } catch {
            print("[Nova] Failed to decode permissions list: \(error)")
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

// MARK: - FFI Function Declarations

@_silgen_name("nova_core_get_extension_title")
func nova_core_get_extension_title(_ handle: OpaquePointer, _ extensionId: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nova_core_check_permissions")
func nova_core_check_permissions(_ handle: OpaquePointer, _ extensionId: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nova_core_grant_permission")
func nova_core_grant_permission(_ handle: OpaquePointer, _ extensionId: UnsafePointer<CChar>, _ permission: UnsafePointer<CChar>) -> Int32

@_silgen_name("nova_core_grant_all_permissions")
func nova_core_grant_all_permissions(_ handle: OpaquePointer, _ extensionId: UnsafePointer<CChar>) -> Int32

@_silgen_name("nova_core_revoke_permission")
func nova_core_revoke_permission(_ handle: OpaquePointer, _ extensionId: UnsafePointer<CChar>, _ permission: UnsafePointer<CChar>) -> Int32

@_silgen_name("nova_core_revoke_all_permissions")
func nova_core_revoke_all_permissions(_ handle: OpaquePointer, _ extensionId: UnsafePointer<CChar>) -> Int32

@_silgen_name("nova_core_list_permissions")
func nova_core_list_permissions(_ handle: OpaquePointer) -> UnsafeMutablePointer<CChar>?
