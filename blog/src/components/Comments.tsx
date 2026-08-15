import { useEffect, useMemo, useState, type CSSProperties, type FormEvent } from 'react';

import './Comments.css';

import { blogApi } from '../lib/api';
import { createSupabaseBrowserClient } from '../lib/auth/supabase';
import type { Comment } from '../lib/models/comments';
import { Button } from './ui/react/Button';
import { Textarea } from './ui/react/Textarea';

interface Props {
	postId: string;
}

interface CommentNodeProps {
	comment: Comment;
	childrenByParent: Map<string | null, Comment[]>;
	currentUserId?: string;
	depth: number;
	replyingTo?: string;
	editingId?: string;
	onReply: (parentCommentId: string, body: string) => Promise<void>;
	onEdit: (commentId: string, body: string) => Promise<void>;
	onDelete: (commentId: string) => Promise<void>;
	setReplyingTo: (id?: string) => void;
	setEditingId: (id?: string) => void;
}

function CommentNode({
	comment,
	childrenByParent,
	currentUserId,
	depth,
	replyingTo,
	editingId,
	onReply,
	onEdit,
	onDelete,
	setReplyingTo,
	setEditingId,
}: CommentNodeProps) {
	const isOwner = comment.authorId === currentUserId;
	const isDeleted = Boolean(comment.deletedAt);
	const children = childrenByParent.get(comment.id) ?? [];

	async function submitReply(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = event.currentTarget;
		const body = new FormData(form).get('body')?.toString().trim();
		if (!body) return;
		await onReply(comment.id, body);
		form.reset();
	}

	async function submitEdit(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const body = new FormData(event.currentTarget).get('body')?.toString().trim();
		if (!body) return;
		await onEdit(comment.id, body);
	}

	return (
		<article className="comment" style={{ '--depth': Math.min(depth, 4) } as CSSProperties}>
			<p className="comment-meta">
				{isOwner ? 'You' : 'Member'} · {new Date(comment.createdAt).toLocaleString()}
			</p>
			{editingId === comment.id ? (
				<form className="comment-form compact" onSubmit={submitEdit}>
					<Textarea name="body" rows={3} defaultValue={comment.body} maxLength={5000} required />
					<div className="comment-actions">
						<Button type="submit">Save</Button>
						<Button type="button" onClick={() => setEditingId()}>Cancel</Button>
					</div>
				</form>
			) : <p className="comment-body">{isDeleted ? '[Comment deleted]' : comment.body}</p>}
			{!isDeleted && editingId !== comment.id && (
				<div className="comment-actions">
					<Button type="button" onClick={() => setReplyingTo(replyingTo === comment.id ? undefined : comment.id)}>Reply</Button>
					{isOwner && <Button type="button" onClick={() => setEditingId(comment.id)}>Edit</Button>}
					{isOwner && <Button type="button" onClick={() => void onDelete(comment.id)}>Delete</Button>}
				</div>
			)}
			{replyingTo === comment.id && (
				<form className="comment-form compact" onSubmit={submitReply}>
					<label>Reply to this comment</label>
					<Textarea name="body" rows={3} maxLength={5000} required autoFocus />
					<div className="comment-actions">
						<Button type="submit">Post reply</Button>
						<Button type="button" onClick={() => setReplyingTo()}>Cancel</Button>
					</div>
				</form>
			)}
			{children.map((child) => (
				<CommentNode
					key={child.id}
					comment={child}
					childrenByParent={childrenByParent}
					currentUserId={currentUserId}
					depth={depth + 1}
					replyingTo={replyingTo}
					editingId={editingId}
					onReply={onReply}
					onEdit={onEdit}
					onDelete={onDelete}
					setReplyingTo={setReplyingTo}
					setEditingId={setEditingId}
				/>
			))}
		</article>
	);
}

export default function Comments({ postId }: Props) {
	const [comments, setComments] = useState<Comment[]>([]);
	const [currentUserId, setCurrentUserId] = useState<string>();
	const [status, setStatus] = useState('Loading comments…');
	const [replyingTo, setReplyingTo] = useState<string>();
	const [editingId, setEditingId] = useState<string>();
	const supabase = useMemo(() => createSupabaseBrowserClient(), []);

	const childrenByParent = useMemo(() => {
		const ids = new Set(comments.map((comment) => comment.id));
		const tree = new Map<string | null, Comment[]>();
		for (const comment of comments) {
			const parentId = comment.parentCommentId && ids.has(comment.parentCommentId) ? comment.parentCommentId : null;
			tree.set(parentId, [...(tree.get(parentId) ?? []), comment]);
		}
		return tree;
	}, [comments]);

	useEffect(() => {
		void (async () => {
			try {
				const [{ data: { user } }, loadedComments] = await Promise.all([
					supabase.auth.getUser(),
					blogApi.listComments(postId),
				]);
				setCurrentUserId(user?.id);
				setComments(loadedComments);
				setStatus(loadedComments.length ? `${loadedComments.length} comment${loadedComments.length === 1 ? '' : 's'}` : '');
			} catch {
				setStatus('Comments could not be loaded. Please try again shortly.');
			}
		})();
	}, [postId, supabase]);

	async function accessToken() {
		const { data: { session } } = await supabase.auth.getSession();
		if (!session) throw new Error('Please sign in from the header before commenting.');
		return session.access_token;
	}

	async function createComment(body: string, parentCommentId: string | null = null) {
		const comment = await blogApi.createComment(postId, { body, parentCommentId }, await accessToken());
		setComments((items) => [...items, comment]);
		setReplyingTo(undefined);
		setStatus('Comment posted.');
	}

	async function updateComment(id: string, body: string) {
		const updated = await blogApi.updateComment(id, { body }, await accessToken());
		setComments((items) => items.map((comment) => comment.id === id ? updated : comment));
		setEditingId(undefined);
		setStatus('Comment updated.');
	}

	async function deleteComment(id: string) {
		if (!window.confirm('Delete this comment? Replies will remain visible.')) return;
		await blogApi.deleteComment(id, await accessToken());
		setComments((items) => items.map((comment) => comment.id === id ? { ...comment, body: '', deletedAt: new Date().toISOString() } : comment));
		setStatus('Comment deleted.');
	}

	async function submitNewComment(event: FormEvent<HTMLFormElement>) {
		event.preventDefault();
		const form = event.currentTarget;
		const body = new FormData(form).get('body')?.toString().trim();
		if (!body) return;
		try {
			await createComment(body);
			form.reset();
		} catch (error) {
			setStatus(error instanceof Error ? error.message : 'Unable to post comment.');
		}
	}

	const roots = childrenByParent.get(null) ?? [];
	return (
		<section className="comments" aria-labelledby="comments-heading">
			<h2 id="comments-heading">Comments</h2>
			<p className="comment-status" aria-live="polite">{status}</p>
			<form className="comment-form" onSubmit={submitNewComment}>
				<label htmlFor="new-comment">Join the discussion</label>
				<Textarea id="new-comment" name="body" rows={4} maxLength={5000} required />
				<Button type="submit">Post comment</Button>
			</form>
			<div className="comment-list">
				{roots.length ? roots.map((comment) => (
					<CommentNode key={comment.id} comment={comment} childrenByParent={childrenByParent} currentUserId={currentUserId} depth={0} replyingTo={replyingTo} editingId={editingId} onReply={createComment} onEdit={updateComment} onDelete={deleteComment} setReplyingTo={setReplyingTo} setEditingId={setEditingId} />
				)) : <p>No comments yet. Start the conversation.</p>}
			</div>
		</section>
	);
}
