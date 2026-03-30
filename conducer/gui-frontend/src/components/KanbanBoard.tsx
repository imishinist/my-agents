import type { Feature, FeatureStatus } from '../types';
import { FeatureCard } from './FeatureCard';

const columns: { key: string; label: string; statuses: FeatureStatus[] }[] = [
  { key: 'pending', label: 'Pending', statuses: ['pending', 'blocked'] },
  { key: 'progress', label: 'In Progress', statuses: ['assigned', 'in_progress'] },
  { key: 'review', label: 'Review', statuses: ['pr_submitted', 'in_review', 'changes_requested'] },
  { key: 'done', label: 'Done', statuses: ['merged'] },
];

export function KanbanBoard({ features }: { features: Feature[] }) {
  return (
    <div className="grid grid-cols-4 gap-4">
      {columns.map((col) => {
        const items = features.filter((f) => col.statuses.includes(f.status));
        return (
          <div key={col.key} className="rounded-lg bg-gray-50 p-3">
            <h3 className="mb-3 text-sm font-bold text-gray-700">
              {col.label}{' '}
              <span className="text-gray-400">({items.length})</span>
            </h3>
            <div className="flex flex-col gap-2">
              {items.map((f) => (
                <FeatureCard key={f.id} feature={f} />
              ))}
            </div>
          </div>
        );
      })}
    </div>
  );
}
