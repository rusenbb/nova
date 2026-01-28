/**
 * Quick Notes - A sample Nova extension
 *
 * Demonstrates:
 * - List view with search filtering
 * - Form for creating new notes
 * - Persistent storage
 * - Actions (copy, delete)
 * - Navigation between views
 */

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
  createActionPanel,
  navigationPush,
  navigationPop,
  registerCallback,
} from "@aspect/nova";

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

interface Note {
  id: string;
  title: string;
  content: string;
  createdAt: number;
}

const STORAGE_KEY = "notes";

// ─────────────────────────────────────────────────────────────────────────────
// Storage Helpers
// ─────────────────────────────────────────────────────────────────────────────

function getNotes(): Note[] {
  return storageGet<Note[]>(STORAGE_KEY) ?? [];
}

function saveNotes(notes: Note[]): void {
  storageSet(STORAGE_KEY, notes);
}

function addNote(title: string, content: string): Note {
  const notes = getNotes();
  const note: Note = {
    id: crypto.randomUUID(),
    title,
    content,
    createdAt: Date.now(),
  };
  notes.unshift(note);
  saveNotes(notes);
  return note;
}

function deleteNote(id: string): void {
  const notes = getNotes().filter((n) => n.id !== id);
  saveNotes(notes);
}

// ─────────────────────────────────────────────────────────────────────────────
// Format Helpers
// ─────────────────────────────────────────────────────────────────────────────

function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - timestamp;

  // Today
  if (diff < 24 * 60 * 60 * 1000 && date.getDate() === now.getDate()) {
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  }

  // This week
  if (diff < 7 * 24 * 60 * 60 * 1000) {
    return date.toLocaleDateString([], { weekday: "short" });
  }

  // Older
  return date.toLocaleDateString([], { month: "short", day: "numeric" });
}

function truncate(str: string, maxLength: number): string {
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - 1) + "…";
}

// ─────────────────────────────────────────────────────────────────────────────
// Search Command - List all notes
// ─────────────────────────────────────────────────────────────────────────────

function NotesListView() {
  const [notes, setNotes] = useState<Note[]>(getNotes);

  const handleDelete = (id: string) => {
    deleteNote(id);
    setNotes(getNotes());
  };

  const handleCopy = (note: Note) => {
    clipboardCopy(`${note.title}\n\n${note.content}`);
    closeWindow();
  };

  const handleCreate = () => {
    // For now, just render the create form directly
    // Navigation between views will be wired up with event handlers
  };

  if (notes.length === 0) {
    return (
      <List searchBarPlaceholder="Search notes...">
        <List.Item
          id="empty"
          title="No notes yet"
          subtitle="Use 'Create Note' command to add your first note"
          icon={Icon.system("doc.text")}
        />
      </List>
    );
  }

  return (
    <List searchBarPlaceholder="Search notes...">
      {notes.map((note) => (
        <List.Item
          id={note.id}
          title={note.title}
          subtitle={truncate(note.content, 60)}
          icon={Icon.system("doc.text")}
          accessories={[Accessory.text(formatDate(note.createdAt))]}
          keywords={[note.content]}
          actions={createActionPanel("Actions", [
            createAction({
              id: "copy",
              title: "Copy to Clipboard",
              icon: Icon.system("doc.on.doc"),
              shortcut: { key: "c", modifiers: ["cmd"] },
              onAction: `copy:${note.id}`,
            }),
            createAction({
              id: "delete",
              title: "Delete Note",
              icon: Icon.system("trash"),
              style: "destructive",
              shortcut: { key: "backspace", modifiers: ["cmd"] },
              onAction: `delete:${note.id}`,
            }),
          ])}
        />
      ))}
    </List>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Create Command - New note form
// ─────────────────────────────────────────────────────────────────────────────

function CreateNoteView() {
  // Register callback for form submission
  const handleSubmit = registerCallback((eventData: { values?: { title?: string; content?: string } }) => {
    const values = eventData.values;
    if (values?.title) {
      addNote(values.title, values.content || "");
      closeWindow();
    }
  });

  return (
    <Form onSubmit={handleSubmit}>
      <Form.TextField
        id="title"
        title="Title"
        placeholder="Note title"
        validation={{ required: true, minLength: 1 }}
      />
      <Form.TextField
        id="content"
        title="Content"
        placeholder="Write your note..."
      />
    </Form>
  );
}

// ─────────────────────────────────────────────────────────────────────────────
// Register Commands
// ─────────────────────────────────────────────────────────────────────────────

registerCommand("search", () => {
  render(<NotesListView />);
});

registerCommand("create", () => {
  render(<CreateNoteView />);
});
