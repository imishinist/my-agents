export interface Epic {
  id: string;
  title: string;
  description: string;
  status: 'draft' | 'active' | 'completed' | 'cancelled' | 'error';
  last_error: string | null;
  created_at: string;
  updated_at: string;
  completed_at: string | null;
}

export interface Feature {
  id: string;
  epic_id: string;
  title: string;
  specification: string;
  status: FeatureStatus;
  worker_id: string | null;
  branch_name: string | null;
  pr_number: number | null;
  depends_on: string; // JSON array
  priority: 'low' | 'medium' | 'high' | 'critical';
  blocked_reason: string | null;
  context_envelope: string | null;
  created_at: string;
  updated_at: string;
}

export type FeatureStatus =
  | 'pending'
  | 'assigned'
  | 'in_progress'
  | 'pr_submitted'
  | 'in_review'
  | 'merged'
  | 'changes_requested'
  | 'blocked'
  | 'cancelled';

export interface Worker {
  id: string;
  runtime_type: string;
  status: 'idle' | 'busy' | 'stalled' | 'offline';
  current_feature_id: string | null;
  worktree_path: string | null;
  pid: number | null;
  last_heartbeat: string | null;
  created_at: string;
  updated_at: string;
}

export interface ActionItem {
  action_type: 'escalation' | 'permission';
  id: string;
  title: string;
  question: string;
  urgency: 'low' | 'medium' | 'high' | 'critical';
  created_at: string;
}

export interface Message {
  id: string;
  correlation_id: string | null;
  source: string;
  destination: string;
  type: string;
  payload: string;
  timestamp: string;
}

export interface SseEvent {
  event_type: string;
  data: string;
}
