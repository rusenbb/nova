import { useEffect, useRef, useImperativeHandle, forwardRef } from "react";

interface SearchInputProps {
  value: string;
  onChange: (value: string) => void;
  onEscape: () => void;
}

export interface SearchInputHandle {
  focus: () => void;
}

export const SearchInput = forwardRef<SearchInputHandle, SearchInputProps>(
  ({ value, onChange, onEscape }, ref) => {
    const inputRef = useRef<HTMLInputElement>(null);

    useImperativeHandle(ref, () => ({
      focus: () => {
        inputRef.current?.focus();
      },
    }));

    useEffect(() => {
      // Auto-focus on mount
      inputRef.current?.focus();
    }, []);

    const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === "Escape") {
        e.preventDefault();
        onChange("");
        onEscape();
      }
    };

    return (
      <input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Search apps, calculate, or type a command..."
        className="w-full bg-transparent text-nova-text text-lg px-5 py-4 outline-none placeholder:text-nova-text-muted"
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
        spellCheck={false}
      />
    );
  }
);
