//
//  HotkeyManager.swift
//  Nova
//
//  Global hotkey handling using CGEvent tap.
//  Requires Accessibility permissions.
//

import Cocoa
import Carbon.HIToolbox

final class HotkeyManager {
    typealias HotkeyHandler = () -> Void

    private var eventTap: CFMachPort?
    private var runLoopSource: CFRunLoopSource?
    private var handler: HotkeyHandler?

    // Default: Option + Space
    private let modifiers: CGEventFlags = .maskAlternate
    private let keyCode: CGKeyCode = CGKeyCode(kVK_Space)

    init() {}

    deinit {
        stop()
    }

    // MARK: - Public API

    func start(handler: @escaping HotkeyHandler) -> Bool {
        self.handler = handler

        // Check accessibility permissions
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue(): true] as CFDictionary
        guard AXIsProcessTrustedWithOptions(options) else {
            print("[Nova] Accessibility permissions not granted")
            return false
        }

        // Create event tap
        let eventMask = (1 << CGEventType.keyDown.rawValue)

        // We need to capture self weakly in the callback
        let callback: CGEventTapCallBack = { (proxy, type, event, refcon) -> Unmanaged<CGEvent>? in
            guard let refcon = refcon else { return Unmanaged.passRetained(event) }

            let manager = Unmanaged<HotkeyManager>.fromOpaque(refcon).takeUnretainedValue()
            return manager.handleEvent(proxy: proxy, type: type, event: event)
        }

        let refcon = Unmanaged.passUnretained(self).toOpaque()

        guard let tap = CGEvent.tapCreate(
            tap: .cgSessionEventTap,
            place: .headInsertEventTap,
            options: .defaultTap,
            eventsOfInterest: CGEventMask(eventMask),
            callback: callback,
            userInfo: refcon
        ) else {
            print("[Nova] Failed to create event tap")
            return false
        }

        eventTap = tap
        runLoopSource = CFMachPortCreateRunLoopSource(kCFAllocatorDefault, tap, 0)

        if let source = runLoopSource {
            CFRunLoopAddSource(CFRunLoopGetCurrent(), source, .commonModes)
            CGEvent.tapEnable(tap: tap, enable: true)
            print("[Nova] Hotkey listener started (Option+Space)")
            return true
        }

        return false
    }

    func stop() {
        if let tap = eventTap {
            CGEvent.tapEnable(tap: tap, enable: false)
        }

        if let source = runLoopSource {
            CFRunLoopRemoveSource(CFRunLoopGetCurrent(), source, .commonModes)
        }

        eventTap = nil
        runLoopSource = nil
        handler = nil
    }

    // MARK: - Private

    private func handleEvent(
        proxy: CGEventTapProxy,
        type: CGEventType,
        event: CGEvent
    ) -> Unmanaged<CGEvent>? {
        // Handle tap disabled events (system can disable taps under load)
        if type == .tapDisabledByTimeout || type == .tapDisabledByUserInput {
            if let tap = eventTap {
                CGEvent.tapEnable(tap: tap, enable: true)
            }
            return Unmanaged.passRetained(event)
        }

        // Only process keyDown events
        guard type == .keyDown else {
            return Unmanaged.passRetained(event)
        }

        let eventKeyCode = CGKeyCode(event.getIntegerValueField(.keyboardEventKeycode))
        let eventFlags = event.flags

        // Check if this is our hotkey (Option + Space)
        let hasOption = eventFlags.contains(.maskAlternate)
        let hasNoOtherModifiers = !eventFlags.contains(.maskCommand) &&
                                   !eventFlags.contains(.maskControl) &&
                                   !eventFlags.contains(.maskShift)

        if eventKeyCode == keyCode && hasOption && hasNoOtherModifiers {
            // Trigger handler on main thread
            DispatchQueue.main.async { [weak self] in
                self?.handler?()
            }
            // Consume the event
            return nil
        }

        return Unmanaged.passRetained(event)
    }
}
