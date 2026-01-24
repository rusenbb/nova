// src/index.tsx
import {
  List,
  Form,
  Icon,
  Accessory,
  registerCommand,
  render,
  useState,
  storageGet,
  storageSet,
  clipboardCopy,
  closeWindow,
  createAction,
  createActionPanel
} from "@aspect/nova";
import { jsx, jsxs } from "@aspect/nova/jsx-runtime";
var STORAGE_KEY = "notes";
function getNotes() {
  return storageGet(STORAGE_KEY) ?? [];
}
function saveNotes(notes) {
  storageSet(STORAGE_KEY, notes);
}
function deleteNote(id) {
  const notes = getNotes().filter((n) => n.id !== id);
  saveNotes(notes);
}
function formatDate(timestamp) {
  const date = new Date(timestamp);
  const now = /* @__PURE__ */ new Date();
  const diff = now.getTime() - timestamp;
  if (diff < 24 * 60 * 60 * 1e3 && date.getDate() === now.getDate()) {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }
  if (diff < 7 * 24 * 60 * 60 * 1e3) {
    return date.toLocaleDateString([], { weekday: "short" });
  }
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}
function truncate(str, maxLength) {
  if (str.length <= maxLength)
    return str;
  return str.slice(0, maxLength - 1) + "\u2026";
}
function NotesListView() {
  const [notes, setNotes] = useState(getNotes);
  const handleDelete = (id) => {
    deleteNote(id);
    setNotes(getNotes());
  };
  const handleCopy = (note) => {
    clipboardCopy(`${note.title}

${note.content}`);
    closeWindow();
  };
  const handleCreate = () => {
  };
  if (notes.length === 0) {
    return /* @__PURE__ */ jsx(List, { searchBarPlaceholder: "Search notes...", children: /* @__PURE__ */ jsx(
      List.Item,
      {
        id: "empty",
        title: "No notes yet",
        subtitle: "Use 'Create Note' command to add your first note",
        icon: Icon.system("doc.text")
      }
    ) });
  }
  return /* @__PURE__ */ jsx(List, { searchBarPlaceholder: "Search notes...", children: notes.map((note) => /* @__PURE__ */ jsx(
    List.Item,
    {
      id: note.id,
      title: note.title,
      subtitle: truncate(note.content, 60),
      icon: Icon.system("doc.text"),
      accessories: [Accessory.text(formatDate(note.createdAt))],
      keywords: [note.content],
      actions: createActionPanel("Actions", [
        createAction({
          id: "copy",
          title: "Copy to Clipboard",
          icon: Icon.system("doc.on.doc"),
          shortcut: { key: "c", modifiers: ["cmd"] },
          onAction: `copy:${note.id}`
        }),
        createAction({
          id: "delete",
          title: "Delete Note",
          icon: Icon.system("trash"),
          style: "destructive",
          shortcut: { key: "backspace", modifiers: ["cmd"] },
          onAction: `delete:${note.id}`
        })
      ])
    }
  )) });
}
function CreateNoteView() {
  return /* @__PURE__ */ jsxs(Form, { onSubmit: "submit", children: [
    /* @__PURE__ */ jsx(
      Form.TextField,
      {
        id: "title",
        title: "Title",
        placeholder: "Note title",
        validation: { required: true, minLength: 1 }
      }
    ),
    /* @__PURE__ */ jsx(
      Form.TextField,
      {
        id: "content",
        title: "Content",
        placeholder: "Write your note..."
      }
    )
  ] });
}
registerCommand("search", (props) => {
  render(() => /* @__PURE__ */ jsx(NotesListView, {}));
});
registerCommand("create", (props) => {
  render(() => /* @__PURE__ */ jsx(CreateNoteView, {}));
});
