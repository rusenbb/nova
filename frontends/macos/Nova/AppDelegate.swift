//
//  AppDelegate.swift
//  Nova
//
//  Main application delegate handling lifecycle and menu bar.
//

import Cocoa

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var statusItem: NSStatusItem?
    private var searchPanel: SearchPanel?
    private var hotkeyManager: HotkeyManager?
    private var core: NovaCore?
    private var clipboardTimer: Timer?

    // MARK: - Application Lifecycle

    func applicationDidFinishLaunching(_ notification: Notification) {
        // Initialize the Rust core
        guard let novaCore = NovaCore() else {
            showAlert(title: "Failed to initialize", message: "Could not initialize Nova core engine.")
            NSApp.terminate(nil)
            return
        }
        core = novaCore

        // Setup UI
        setupStatusItem()
        setupSearchPanel()
        setupHotkey()
        setupClipboardPolling()

        print("[Nova] Application launched successfully")
    }

    func applicationWillTerminate(_ notification: Notification) {
        clipboardTimer?.invalidate()
        hotkeyManager?.stop()
        core = nil
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows flag: Bool) -> Bool {
        togglePanel()
        return true
    }

    // MARK: - Setup

    private func setupStatusItem() {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)

        if let button = statusItem?.button {
            button.image = NSImage(systemSymbolName: "sparkle.magnifyingglass", accessibilityDescription: "Nova")
            button.action = #selector(statusItemClicked)
            button.target = self
        }

        let menu = NSMenu()
        menu.addItem(NSMenuItem(title: "Show Nova", action: #selector(showPanel), keyEquivalent: " "))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Reload", action: #selector(reloadConfig), keyEquivalent: "r"))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Quit Nova", action: #selector(quitApp), keyEquivalent: "q"))

        statusItem?.menu = menu
    }

    private func setupSearchPanel() {
        searchPanel = SearchPanel(
            contentRect: NSRect(x: 0, y: 0, width: 620, height: 400),
            styleMask: [],
            backing: .buffered,
            defer: false
        )

        searchPanel?.onSearch = { [weak self] query in
            return self?.core?.search(query: query, maxResults: 10) ?? []
        }

        searchPanel?.onExecute = { [weak self] index in
            return self?.core?.execute(index: index) ?? .error
        }

        searchPanel?.onHide = {
            // Could track analytics or state here
        }
    }

    private func setupHotkey() {
        hotkeyManager = HotkeyManager()

        let success = hotkeyManager?.start { [weak self] in
            self?.togglePanel()
        }

        if success != true {
            print("[Nova] Hotkey registration failed - accessibility permissions may be needed")
        }
    }

    private func setupClipboardPolling() {
        // Poll clipboard every 500ms
        clipboardTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { [weak self] _ in
            self?.core?.pollClipboard()
        }
    }

    // MARK: - Actions

    @objc private func statusItemClicked() {
        // Right-click shows menu, left-click toggles panel
        if let event = NSApp.currentEvent, event.type == .rightMouseUp {
            statusItem?.menu?.popUp(positioning: nil, at: NSPoint.zero, in: statusItem?.button)
        } else {
            togglePanel()
        }
    }

    @objc private func showPanel() {
        searchPanel?.show()
    }

    @objc private func togglePanel() {
        if searchPanel?.isVisible == true {
            searchPanel?.hide()
        } else {
            searchPanel?.show()
        }
    }

    @objc private func reloadConfig() {
        core?.reload()
        print("[Nova] Configuration reloaded")
    }

    @objc private func quitApp() {
        NSApp.terminate(nil)
    }

    // MARK: - Helpers

    private func showAlert(title: String, message: String) {
        let alert = NSAlert()
        alert.messageText = title
        alert.informativeText = message
        alert.alertStyle = .critical
        alert.runModal()
    }
}
