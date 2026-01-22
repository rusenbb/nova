//
//  NovaApp.swift
//  Nova
//
//  Main entry point for the Nova macOS application.
//

import Cocoa

@main
struct NovaApp {
    static func main() {
        let app = NSApplication.shared
        let delegate = AppDelegate()
        app.delegate = delegate

        // Hide dock icon - Nova is a menu bar app
        app.setActivationPolicy(.accessory)

        app.run()
    }
}
