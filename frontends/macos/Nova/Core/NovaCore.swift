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
}
