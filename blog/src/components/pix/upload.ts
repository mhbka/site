import type { Pix } from '../../lib/models/pix.ts';

export interface PixUploadProgress {
	uploaded: number;
	total: number;
	failed: number;
}

interface UploadPixFilesOptions {
	files: Iterable<File>;
	tags: string[];
	upload: (file: File, tags: string[]) => Promise<Pix>;
	onProgress: (progress: PixUploadProgress) => void;
}

// Uploads each selected file and reports the batch outcome.
export async function uploadPixFiles({ files, tags, upload, onProgress }: UploadPixFilesOptions): Promise<PixUploadProgress> {
	const queuedFiles = [...files];
	const progress: PixUploadProgress = { uploaded: 0, total: queuedFiles.length, failed: 0 };
	onProgress(progress);

	for (const file of queuedFiles) {
		try {
			await upload(file, tags);
			progress.uploaded += 1;
		} catch {
			progress.failed += 1;
		}
		onProgress(progress);
	}

	return progress;
}
