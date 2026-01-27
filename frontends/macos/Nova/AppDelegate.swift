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
    private var permissionsWindowController: PermissionsManagerWindowController?

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
            button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        }

        // Don't assign menu directly - we'll show it on right-click only
    }

    private func createMenu() -> NSMenu {
        let menu = NSMenu()
        menu.addItem(NSMenuItem(title: "Show Nova (Option+Space)", action: #selector(showPanel), keyEquivalent: ""))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Manage Permissions...", action: #selector(showPermissions), keyEquivalent: ""))
        menu.addItem(NSMenuItem(title: "Reload", action: #selector(reloadConfig), keyEquivalent: "r"))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(NSMenuItem(title: "Quit Nova", action: #selector(quitApp), keyEquivalent: "q"))
        return menu
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
            return self?.core?.execute(index: index) ?? .error("Core not initialized")
        }

        searchPanel?.onExecuteExtension = { [weak self] extensionId, commandId, argument in
            return self?.core?.executeExtension(extensionId: extensionId, commandId: commandId, argument: argument)
        }

        searchPanel?.onExtensionEvent = { [weak self] extensionId, callbackId, payload in
            return self?.core?.sendEvent(extensionId: extensionId, callbackId: callbackId, payload: payload)
        }

        searchPanel?.onCheckPermissions = { [weak self] extensionId, commandId, completion in
            guard let core = self?.core else {
                completion(true) // Allow if no core
                return
            }

            // Get extension title for the dialog
            let title = extensionId // TODO: Get from manifest

            core.showPermissionConsentIfNeeded(
                extensionId: extensionId,
                extensionTitle: title,
                completion: completion
            )
        }

        searchPanel?.onHide = {
            // Could track analytics or state here
        }
    }

    private func setupHotkey() {
        hotkeyManager = HotkeyManager()

        let success = hotkeyManager?.start { [weak self] in
            print("[Nova] Hotkey triggered!")
            self?.togglePanel()
        }

        if success == true {
            print("[Nova] Hotkey registered successfully (Option+Space)")
        } else {
            print("[Nova] Hotkey registration FAILED")
            print("[Nova] Please grant Accessibility permissions:")
            print("[Nova]   System Settings → Privacy & Security → Accessibility → Enable Nova")
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
        guard let event = NSApp.currentEvent, let button = statusItem?.button else { return }

        if event.type == .rightMouseUp {
            // Right-click shows menu
            let menu = createMenu()
            menu.popUp(positioning: nil, at: NSPoint(x: 0, y: button.bounds.height + 5), in: button)
        } else {
            // Left-click toggles panel
            togglePanel()
        }
    }

    @objc private func showPanel() {
        searchPanel?.show()
    }

    @objc private func togglePanel() {
        guard let panel = searchPanel else { return }

        if panel.isPanelVisible {
            print("[Nova] Hiding panel")
            panel.hide()
        } else {
            print("[Nova] Showing panel")
            panel.show()
        }
    }

    @objc private func reloadConfig() {
        core?.reload()
        print("[Nova] Configuration reloaded")
    }

    @objc private func showPermissions() {
        guard let core = core else { return }

        if permissionsWindowController == nil {
            permissionsWindowController = PermissionsManagerWindowController(core: core)
        }
        permissionsWindowController?.showWindow()
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
