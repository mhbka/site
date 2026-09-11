/** Represents a public image in the pix gallery. */
export interface Pix {
	id: string;
	publicUrl: string;
	tags: string[];
	createdAt: string;
}

/** Represents one cursor-paginated pix gallery response. */
export interface PixPage {
	images: Pix[];
	nextBefore: string | null;
	hasMore: boolean;
}

/** Contains the presigned upload details for a new pix image. */
export interface PixUpload {
	imageId: string;
	uploadUrl: string;
	publicUrl: string;
}
