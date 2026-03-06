// stores/messaging.ts — Frontend API for the Universal Messaging System
// Source of truth: 00_MASTER_GUIDE.md §6, commands/messaging.rs
//
// All business logic lives in Rust — this file only mirrors the Tauri command interface.
// Covers all channels: general, security, medical_heads, broadcast.
//
// Raw types match Rust serialization. View types add convenience fields.

import { invoke } from '@tauri-apps/api/core';
import { writable } from 'svelte/store';

// ── Raw types (match Rust structs exactly) ────────────────────────────────────

interface RawInboxMessage {
  id: string;
  from_id: string;
  from_name: string;
  subject: string;
  channel: string;
  recipient_type: string;
  read_at: string | null;
  recalled_at: string | null;
  scheduled_at: string | null;
  created_at: string;
}

interface RawMessageSummary {
  id: string;
  from_id: string;
  from_name: string;
  subject: string;
  body: string;
  channel: string;
  scheduled_at: string | null;
  recalled_at: string | null;
  created_at: string;
}

interface RawMessageRecipient {
  user_id: string;
  full_name: string;
  recipient_type: string;
  read_at: string | null;
}

interface RawAttachmentSummary {
  id: string;
  original_filename: string;
  content_type: string;
  storage_path: string;
  file_size_bytes: number | null;
  uploaded_at: string;
}

interface RawMessageDetail {
  message: RawMessageSummary;
  recipients: RawMessageRecipient[];
  attachments: RawAttachmentSummary[];
}

// ── View types (used by Svelte components) ────────────────────────────────────

/** Inbox message row — includes convenience fields. */
export interface InboxMessage {
  id: string;
  sender_name: string;
  subject: string;
  channel: string;
  body_preview: string;
  sent_at: string;
  read_at: string | null;
  recalled: boolean;
}

/** Sent message row. */
export interface MessageSummary {
  id: string;
  subject: string;
  body_preview: string;
  sent_at: string;
  recalled: boolean;
}

/** Attachment info. */
export interface AttachmentInfo {
  id: string;
  file_name: string;
  content_type: string;
  storage_path: string;
  file_size_bytes: number | null;
}

/** Recipient in detail view. */
export interface RecipientInfo {
  user_id: string;
  full_name: string;
  recipient_type: string;
  read_at: string | null;
}

/** Full message detail (flattened for view convenience). */
export interface MessageDetail {
  id: string;
  sender_name: string;
  subject: string;
  body: string;
  channel: string;
  sent_at: string;
  recalled: boolean;
  recipients_to: RecipientInfo[];
  recipients_cc: RecipientInfo[];
  recipients_bcc: RecipientInfo[];
  attachments: AttachmentInfo[];
}

export interface UnreadCount {
  channel: string;
  count: number;
}

export interface EligibleRecipient {
  id: string;
  full_name: string;
  username: string;
  role_name: string;
}

export interface GroupSummary {
  id: string;
  name: string;
  description: string | null;
  member_count: number;
  created_by: string;
  created_at: string;
}

export interface GroupMember {
  user_id: string;
  full_name: string;
  role_name: string;
  added_at: string;
}

export interface SendMessagePayload {
  subject: string;
  body: string;
  channel: string;
  recipients_to: string[];
  recipients_cc?: string[];
  recipients_bcc?: string[];
  scheduled_at?: string;
  group_ids?: string[];
}

// ── View adapters ─────────────────────────────────────────────────────────────

function truncate(s: string, max = 100): string {
  return s.length > max ? s.slice(0, max) + '…' : s;
}

function toInboxMessage(raw: RawInboxMessage): InboxMessage {
  return {
    id: raw.id,
    sender_name: raw.from_name,
    subject: raw.subject,
    channel: raw.channel,
    body_preview: '', // inbox query doesn't include body
    sent_at: raw.created_at,
    read_at: raw.read_at,
    recalled: !!raw.recalled_at,
  };
}

function toMessageSummary(raw: RawMessageSummary): MessageSummary {
  return {
    id: raw.id,
    subject: raw.subject,
    body_preview: truncate(raw.body),
    sent_at: raw.created_at,
    recalled: !!raw.recalled_at,
  };
}

function toAttachmentInfo(raw: RawAttachmentSummary): AttachmentInfo {
  return {
    id: raw.id,
    file_name: raw.original_filename,
    content_type: raw.content_type,
    storage_path: raw.storage_path,
    file_size_bytes: raw.file_size_bytes,
  };
}

function toMessageDetail(raw: RawMessageDetail): MessageDetail {
  return {
    id: raw.message.id,
    sender_name: raw.message.from_name,
    subject: raw.message.subject,
    body: raw.message.body,
    channel: raw.message.channel,
    sent_at: raw.message.created_at,
    recalled: !!raw.message.recalled_at,
    recipients_to: raw.recipients.filter((r) => r.recipient_type === 'to'),
    recipients_cc: raw.recipients.filter((r) => r.recipient_type === 'cc'),
    recipients_bcc: raw.recipients.filter((r) => r.recipient_type === 'bcc'),
    attachments: raw.attachments.map(toAttachmentInfo),
  };
}

// ── Channel definitions ───────────────────────────────────────────────────────

export type Channel = 'general' | 'security' | 'medical_heads' | 'broadcast';

export const CHANNEL_LABELS: Record<Channel, string> = {
  general: 'General',
  security: 'Security Line',
  medical_heads: 'Medical Heads',
  broadcast: 'Broadcast',
};

// ── Messaging API ─────────────────────────────────────────────────────────────

/** Send a message on any channel. */
export async function sendMessage(payload: SendMessagePayload): Promise<string> {
  return invoke<string>('send_message', { payload });
}

/** Get inbox messages for a specific channel. */
export async function getInbox(channel: Channel): Promise<InboxMessage[]> {
  const raw = await invoke<RawInboxMessage[]>('get_inbox', { channel });
  return raw.map(toInboxMessage);
}

/** Get sent messages for a specific channel. */
export async function getSentMessages(channel: Channel): Promise<MessageSummary[]> {
  const raw = await invoke<RawMessageSummary[]>('get_sent_messages', { channel });
  return raw.map(toMessageSummary);
}

/** Get full message detail (recipients, attachments). */
export async function getMessageDetail(messageId: string): Promise<MessageDetail> {
  const raw = await invoke<RawMessageDetail>('get_message_detail', { messageId });
  return toMessageDetail(raw);
}

/** Mark a message as read. */
export async function msgMarkRead(messageId: string): Promise<void> {
  await invoke('msg_mark_read', { messageId });
}

/** Recall a message (only if unread by all recipients). */
export async function msgRecall(messageId: string): Promise<void> {
  await invoke('msg_recall', { messageId });
}

/** Soft-delete a message. */
export async function deleteMessage(messageId: string): Promise<void> {
  await invoke('delete_message', { messageId });
}

/** Get unread counts per channel for the current user. */
export async function getUnreadCounts(): Promise<UnreadCount[]> {
  return invoke<UnreadCount[]>('get_unread_counts');
}

// ── Shared unread counts store (reactive across layout + pages) ──────────────

/** Writable store so the layout sidebar can reactively update. */
export const unreadCountsStore = writable<UnreadCount[]>([]);

/** Refresh the shared unread counts store (call after marking a message read). */
export async function refreshUnreadCounts(): Promise<void> {
  try {
    const counts = await getUnreadCounts();
    unreadCountsStore.set(counts);
  } catch { /* ignore — best effort */ }
}

/** Get eligible recipients for a channel (used in compose picker). */
export async function getEligibleRecipients(channel: Channel): Promise<EligibleRecipient[]> {
  return invoke<EligibleRecipient[]>('get_eligible_recipients', { channel });
}

// ── Group API ─────────────────────────────────────────────────────────────────

/** Create a messaging group. */
export async function createMessagingGroup(
  name: string,
  description?: string,
): Promise<string> {
  return invoke<string>('create_messaging_group', { name, description });
}

/** List all messaging groups visible to the current user. */
export async function getMessagingGroups(): Promise<GroupSummary[]> {
  return invoke<GroupSummary[]>('get_messaging_groups');
}

/** Get members of a messaging group. */
export async function getGroupMembers(groupId: string): Promise<GroupMember[]> {
  return invoke<GroupMember[]>('get_group_members', { groupId });
}

/** Add a member to a messaging group. */
export async function addGroupMember(groupId: string, userId: string): Promise<void> {
  await invoke('add_group_member', { groupId, userId });
}

/** Remove a member from a messaging group. */
export async function removeGroupMember(groupId: string, userId: string): Promise<void> {
  await invoke('remove_group_member', { groupId, userId });
}

/** Soft-delete a messaging group. */
export async function deleteMessagingGroup(groupId: string): Promise<void> {
  await invoke('delete_messaging_group', { groupId });
}
