import type { MilkdownPlugin } from '@milkdown/kit/ctx';
import { uploadConfig } from '@milkdown/kit/plugin/upload';
import { Decoration } from '@milkdown/kit/prose/view';

type UploadImageFile = (file: File) => Promise<string>;

export function imageUploadPlugin(uploadImageFile: UploadImageFile): MilkdownPlugin {
	return (ctx) => {
		ctx.set(uploadConfig.key, {
			enableHtmlFileUploader: false,
			uploadWidgetFactory: (position, spec) => {
				const indicator = document.createElement('span');
				indicator.className = 'image-uploading';
				indicator.textContent = 'Uploading image…';
				return Decoration.widget(position, indicator, spec);
			},
			uploader: async (files, schema) => {
				const image = schema.nodes.image;
				if (!image) throw new Error('The editor image extension is unavailable.');

				const uploadedImages = await Promise.all(
					Array.from(files)
						.filter((file) => file.type.startsWith('image/'))
						.map(async (file) => image.create({
							alt: file.name,
							src: await uploadImageFile(file),
						})),
				);

				if (!uploadedImages.length) throw new Error('Only image files can be uploaded.');
				return uploadedImages;
			},
		});
		return () => {};
	};
}
