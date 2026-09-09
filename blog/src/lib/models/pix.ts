export interface Pix {
	id: string;
	publicUrl: string;
	tags: string[];
	createdAt: string;
}

export interface PixPage {
	images: Pix[];
	nextBefore: string | null;
	hasMore: boolean;
}

export interface PixUpload {
	imageId: string;
	uploadUrl: string;
	publicUrl: string;
}
