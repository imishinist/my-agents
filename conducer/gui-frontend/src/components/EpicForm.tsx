import { useState } from 'react';

interface Props {
  onSubmit: (title: string, description: string) => void;
}

export function EpicForm({ onSubmit }: Props) {
  const [open, setOpen] = useState(false);
  const [title, setTitle] = useState('');
  const [desc, setDesc] = useState('');

  const handleSubmit = () => {
    if (!title.trim()) return;
    onSubmit(title, desc);
    setTitle('');
    setDesc('');
    setOpen(false);
  };

  if (!open) {
    return (
      <button
        className="rounded-lg bg-blue-600 px-4 py-2 text-sm font-medium text-white hover:bg-blue-700"
        onClick={() => setOpen(true)}
      >
        + New Epic
      </button>
    );
  }

  return (
    <div className="rounded-lg border border-gray-200 bg-white p-4">
      <input
        className="mb-2 w-full rounded border border-gray-300 px-3 py-2 text-sm"
        placeholder="Epic title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
      />
      <textarea
        className="mb-2 w-full rounded border border-gray-300 px-3 py-2 text-sm"
        rows={3}
        placeholder="Description (what do you want to build?)"
        value={desc}
        onChange={(e) => setDesc(e.target.value)}
      />
      <div className="flex gap-2">
        <button
          className="rounded bg-blue-600 px-4 py-1.5 text-sm text-white hover:bg-blue-700"
          onClick={handleSubmit}
        >
          Create
        </button>
        <button
          className="rounded px-4 py-1.5 text-sm text-gray-600 hover:bg-gray-100"
          onClick={() => setOpen(false)}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
