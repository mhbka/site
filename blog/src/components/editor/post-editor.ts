import { Crepe } from '@milkdown/crepe';
import '@milkdown/crepe/theme/common/style.css';
import '@milkdown/crepe/theme/nord.css';

import { blogApi } from '../../lib/api.ts';
import { createSupabaseBrowserClient } from '../../lib/auth/supabase.ts';
import { uploadImage } from '../../lib/image-upload.ts';
import { addTag, normalizeTag } from '../../lib/tags.ts';

/** Initializes the Milkdown editor and its post form controls. */
async function initPostEditor() {
	const form = document.querySelector<HTMLFormElement>('[data-post-editor]');
	const root = document.querySelector<HTMLElement>('#post-content');
	const title = document.querySelector<HTMLInputElement>('#post-title');
	const slug = document.querySelector<HTMLInputElement>('#post-slug');
	const tagsInput = document.querySelector<HTMLInputElement>('#post-tags');
	const tagList = document.querySelector<HTMLUListElement>('[data-tag-list]');
	const tagSuggestions = document.querySelector<HTMLDataListElement>('#post-tag-suggestions');
	const button = form?.querySelector<HTMLButtonElement>('button[type="submit"]');
	const status = document.querySelector<HTMLElement>('[data-editor-status]');

	if (!form || !root || !title || !slug || !tagsInput || !tagList || !button || !status) {
		throw new Error('Post editor is missing required elements.');
	}

	const slugInput = slug;
	const selectedTagsInput = tagsInput;
	const selectedTagList = tagList;
	const mediaApiUrl = `${import.meta.env.BACKEND_URL.replace(/\/$/, '')}/media/uploads`;
	const supabaseClient = createSupabaseBrowserClient();
	let postId = form.dataset.postId;
	let tags: string[] = JSON.parse(form.dataset.initialTags ?? '[]');

	/** Returns a validation message for an invalid custom slug. */
	function slugError(value: string) {
		return value && !/^[A-Za-z0-9-]+$/.test(value)
			? 'Slug may contain only letters, numbers, and hyphens.'
			: '';
	}

	/** Applies custom slug validation to the form field. */
	function validateSlug() {
		const error = slugError(slugInput.value);
		slugInput.setCustomValidity(error);
		slugInput.setAttribute('aria-invalid', String(Boolean(error)));
		return error;
	}

	/** Renders the selected tag buttons. */
	function renderTags() {
		selectedTagList.replaceChildren(...tags.map((tag) => {
			const item = document.createElement('li');
			const removeButton = document.createElement('button');
			removeButton.type = 'button';
			removeButton.className = 'ui-button tag';
			removeButton.setAttribute('aria-label', `Remove ${tag} tag`);
			removeButton.dataset.tag = tag;
			removeButton.textContent = `${tag} ×`;
			item.append(removeButton);
			return item;
		}));
	}

	/** Adds the current input value to the selected tags. */
	function commitTag() {
		if (!selectedTagsInput.value) return;
		tags = addTag(tags, selectedTagsInput.value);
		selectedTagsInput.value = '';
		renderTags();
	}

	/** Uploads an editor image after checking the draft and session. */
	async function uploadImageFile(file: File): Promise<string> {
		if (!file.type.startsWith('image/')) throw new Error('Only image files can be uploaded.');
		if (!postId) throw new Error('Save this draft before uploading images.');

		const { data: { session } } = await supabaseClient.auth.getSession();
		if (!session) throw new Error('Please log in before uploading an image.');
		return uploadImage(file, mediaApiUrl, postId, session.access_token);
	}

	slug.addEventListener('input', validateSlug);
	renderTags();

	void blogApi.listTags().then((availableTags) => {
		tagSuggestions?.replaceChildren(...availableTags.map(({ tag, count }) => {
			const option = document.createElement('option');
			option.value = tag;
			option.label = `${count} published post${count === 1 ? '' : 's'}`;
			return option;
		}));
	}).catch(() => {
		// Tag suggestions are optional; authors can always add a new tag.
	});

	tagsInput.addEventListener('input', () => {
		tagsInput.value = normalizeTag(tagsInput.value);
	});
	tagsInput.addEventListener('keydown', (event) => {
		if (event.key === ' ' || event.key === 'Enter') {
			event.preventDefault();
			commitTag();
		}
		if (event.key === 'Backspace' && !tagsInput.value && tags.length) {
			tags = tags.slice(0, -1);
			renderTags();
		}
	});
	tagList.addEventListener('click', (event) => {
		const removeButton = (event.target as HTMLElement).closest<HTMLButtonElement>('[data-tag]');
		if (!removeButton?.dataset.tag) return;
		tags = tags.filter((tag) => tag !== removeButton.dataset.tag);
		renderTags();
		tagsInput.focus();
	});

	// Crepe provides the standard editing controls while preserving Markdown output.
	const editor = new Crepe({
		root,
		defaultValue: root.dataset.initialContent ?? '',
		featureConfigs: {
			[Crepe.Feature.ImageBlock]: {
				onUpload: uploadImageFile,
			},
			[Crepe.Feature.Placeholder]: {
				text: 'Content',
				mode: 'doc',
			},
		},
	});

	await editor.create();

	form.addEventListener('submit', async (event) => {
		event.preventDefault();
		commitTag();
		const contentMd = editor.getMarkdown().trim();
		const postTitle = title.value.trim();
		const postSlug = slug.value.trim();

		if (!postTitle || !contentMd) {
			status.textContent = 'A title and content are required.';
			return;
		}
		if (validateSlug()) {
			slug.reportValidity();
			return;
		}

		button.disabled = true;
		status.textContent = postId ? 'Saving changes...' : 'Creating draft...';
		try {
			const { data: { session } } = await supabaseClient.auth.getSession();
			if (!session) throw new Error('Please log in before saving a post.');
			if (postId) {
				await blogApi.updatePost(postId, { title: postTitle, contentMd, slug: postSlug, tags }, session.access_token);
				status.textContent = 'Changes saved.';
			} else {
				const post = await blogApi.createPost({ title: postTitle, contentMd, slug: postSlug, tags }, session.access_token);
				postId = post.id;
				history.replaceState(null, '', `/posts/edit/${encodeURIComponent(post.id)}`);
				status.textContent = 'Draft created.';
			}
		} catch (error) {
			status.textContent = error instanceof Error ? error.message : 'Unable to save the post. Please try again.';
		} finally {
			button.disabled = false;
		}
	});
}

void initPostEditor();
