import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { SearchInput, SearchInputHandle } from "./components/SearchInput";
import "./App.css";

function App() {
  const [query, setQuery] = useState("");
  const searchInputRef = useRef<SearchInputHandle>(null);

  useEffect(() => {
    const unlistenShown = listen("window-shown", () => {
      setTimeout(() => {
        searchInputRef.current?.focus();
      }, 50);
    });

    const handleGlobalKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        setQuery("");
        invoke("cmd_hide_window");
      }
    };
    document.addEventListener("keydown", handleGlobalKeyDown);

    return () => {
      unlistenShown.then((fn) => fn());
      document.removeEventListener("keydown", handleGlobalKeyDown);
    };
  }, []);

  const handleEscape = () => {
    setQuery("");
    invoke("cmd_hide_window");
  };

  return (
    <main className="w-full h-full nova-glass rounded-xl overflow-hidden">
      <div className="border-b border-white/10">
        <SearchInput
          ref={searchInputRef}
          value={query}
          onChange={setQuery}
          onEscape={handleEscape}
        />
      </div>

      <div className="p-2 text-nova-text-muted text-sm">
        {query ? (
          <p className="px-3 py-6 text-center opacity-60">No results for "{query}"</p>
        ) : (
          <p className="px-3 py-6 text-center opacity-40">Type to search...</p>
        )}
      </div>
    </main>
  );
}

export default App;
