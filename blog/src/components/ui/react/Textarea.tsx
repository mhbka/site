import type { TextareaHTMLAttributes } from 'react';

type Props = TextareaHTMLAttributes<HTMLTextAreaElement>;

/** Renders a consistently styled native textarea. */
export function Textarea({ className, ...props }: Props) {
	return <textarea className={['ui-textarea', className].filter(Boolean).join(' ')} {...props} />;
}
