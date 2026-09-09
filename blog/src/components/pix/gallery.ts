import { blogApi, type Pix } from '../../lib/api.ts';
import { uploadPixFiles } from './upload.ts';

type PixImage = Pick<Pix, 'publicUrl' | 'tags' | 'createdAt'>;

export function initPixGallery(gallery: HTMLElement) {
	const grid = gallery.querySelector<HTMLElement>('[data-moe-grid]');
	const loadMore = gallery.querySelector<HTMLButtonElement>('[data-moe-load-more]');
	const portal = gallery.querySelector<HTMLDialogElement>('[data-moe-portal]');
	const portalImage = gallery.querySelector<HTMLImageElement>('[data-moe-portal-image]');
	const portalCreatedAt = gallery.querySelector<HTMLTimeElement>('[data-moe-portal-created-at]');
	const portalTags = gallery.querySelector<HTMLElement>('[data-moe-portal-tags]');
	const status = gallery.querySelector<HTMLElement>('[data-moe-status]');
	const uploader = gallery.querySelector<HTMLElement>('[data-moe-upload]');
	const dropzone = gallery.querySelector<HTMLLabelElement>('[data-moe-dropzone]');
	const fileInput = gallery.querySelector<HTMLInputElement>('#moe-image');
	const tagsInput = gallery.querySelector<HTMLInputElement>('[data-tag-value]');

	function createThumbnail(pix: PixImage) {
		const button = document.createElement('button');
		button.className = 'ui-button moe-thumbnail';
		button.type = 'button';
		button.ariaLabel = 'view full image';
		button.dataset.imageUrl = pix.publicUrl;
		button.dataset.imageTags = JSON.stringify(pix.tags);
		button.dataset.imageCreatedAt = pix.createdAt;
		const thumbnailImage = document.createElement('img');
		thumbnailImage.src = pix.publicUrl;
		thumbnailImage.alt = '';
		thumbnailImage.loading = 'lazy';
		button.append(thumbnailImage);
		return button;
	}

	function showImage(image: PixImage) {
		if (!portalImage) return;
		portalImage.src = image.publicUrl;
		const createdAt = new Date(image.createdAt);
		if (portalCreatedAt) {
			portalCreatedAt.dateTime = image.createdAt;
			portalCreatedAt.textContent = Number.isNaN(createdAt.getTime()) ? '' : new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'short' }).format(createdAt);
		}
		if (portalTags) portalTags.textContent = image.tags.map((tag) => `#${tag}`).join(' ');
	}

	grid?.addEventListener('click', (event) => {
		const thumbnail = (event.target as Element).closest<HTMLButtonElement>('[data-image-url]');
		if (!thumbnail || !portal || !portalImage) return;
		showImage({
			publicUrl: thumbnail.dataset.imageUrl ?? '',
			tags: JSON.parse(thumbnail.dataset.imageTags ?? '[]') as string[],
			createdAt: thumbnail.dataset.imageCreatedAt ?? '',
		});
		portal.showModal();
	});
	gallery.querySelector('[data-moe-close]')?.addEventListener('click', () => portal?.close());
	portal?.addEventListener('click', (event) => { if (event.target === portal) portal.close(); });

	loadMore?.addEventListener('click', async () => {
		if (!grid || gallery.dataset.hasMore !== 'true' || !gallery.dataset.nextBefore) return;
		loadMore.disabled = true;
		try {
			const page = await blogApi.listPix(60, gallery.dataset.nextBefore, gallery.dataset.tag || undefined);
			page.images.forEach((image) => grid.append(createThumbnail(image)));
			gallery.dataset.nextBefore = page.nextBefore ?? '';
			gallery.dataset.hasMore = String(page.hasMore);
			loadMore.hidden = !page.hasMore;
		} catch { window.alert('could not load more pix.'); }
		finally { loadMore.disabled = false; }
	});

	async function uploadFiles(files: Iterable<File>) {
		if (!grid) return;
		const selectedFiles = [...files];
		if (!selectedFiles.length) return;
		const token = gallery.dataset.token;
		const tags: string[] = JSON.parse(tagsInput?.value ?? '[]');
		if (!tags.length) {
			window.alert('each pix must have at least 1 tag.');
			if (fileInput) fileInput.value = '';
			return;
		}
		if (!token || fileInput?.disabled) return;
		if (fileInput) fileInput.disabled = true;
		uploader?.setAttribute('aria-busy', 'true');
		try {
			const result = await uploadPixFiles({
				files: selectedFiles,
				tags,
				upload: (file, imageTags) => blogApi.uploadPix(file, imageTags, token),
				onProgress: ({ uploaded, total }) => {
					if (status) status.textContent = `${uploaded}/${total} pix uploaded...`;
				},
			});
			if (fileInput) fileInput.value = '';
			if (!result.failed) {
				window.location.reload();
				return;
			}
			if (status) status.textContent = `${result.uploaded}/${result.total} pix uploaded. ${result.failed} failed.`;
		} catch { if (status) status.textContent = 'upload failed. please try again.'; }
		finally {
			if (fileInput) fileInput.disabled = false;
			uploader?.removeAttribute('aria-busy');
		}
	}

	fileInput?.addEventListener('change', () => {
		if (fileInput.files?.length) void uploadFiles(fileInput.files);
	});
	for (const eventName of ['dragenter', 'dragover']) {
		dropzone?.addEventListener(eventName, (event) => {
			event.preventDefault();
			dropzone.classList.add('is-dragging');
		});
	}
	for (const eventName of ['dragleave', 'drop']) {
		dropzone?.addEventListener(eventName, (event) => {
			event.preventDefault();
			dropzone.classList.remove('is-dragging');
		});
	}
	dropzone?.addEventListener('drop', (event) => {
		if (event.dataTransfer?.files.length) void uploadFiles(event.dataTransfer.files);
	});
}
