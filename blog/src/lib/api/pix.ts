import type { ApiRequest } from '../models/api.ts';
import type { Pix, PixPage, PixTagUpdate, PixUpload } from '../models/pix.ts';

/** Creates requests for browsing and uploading pix images. */
export function createPixApi(request: ApiRequest, upload: typeof globalThis.fetch = globalThis.fetch) {
	return {
		getPixStatus: (token: string) => request<{ isPix: boolean }>('/users/is-pix', {}, token),
		listPix: (limit = 60, before?: string, tag?: string) => {
			const query = new URLSearchParams({ limit: String(limit) });
			if (before) query.set('before', before);
			if (tag) query.set('tag', tag);
			return request<PixPage>(`/pix?${query}`);
		},
		listPixTags: () => request<Array<{ tag: string; count: number }>>('/pix/tags'),
		updatePixTags: (imageIds: string[], operation: 'add' | 'remove' | 'overwrite', tags: string[], token: string) =>
			request<PixTagUpdate[]>('/pix/tags', { method: 'PATCH', body: JSON.stringify({ imageIds, operation, tags }) }, token),
		createPixUpload: (contentType: string, tags: string[], token: string) =>
			request<PixUpload>('/pix/uploads', { method: 'POST', body: JSON.stringify({ contentType, tags }) }, token),
		completePixUpload: (id: string, token: string) =>
			request<Pix>(`/pix/uploads/${encodeURIComponent(id)}/complete`, { method: 'POST' }, token),
		uploadPix: async (file: Blob, tags: string[], token: string) => {
			const prepared = await request<PixUpload>(
				'/pix/uploads',
				{ method: 'POST', body: JSON.stringify({ contentType: file.type, tags }) },
				token,
			);
			const response = await upload(prepared.uploadUrl, {
				method: 'PUT',
				headers: { 'Content-Type': file.type },
				body: file,
			});
			if (!response.ok) throw new Error('Pix upload failed');
			return request<Pix>(`/pix/uploads/${encodeURIComponent(prepared.imageId)}/complete`, { method: 'POST' }, token);
		},
	};
}
