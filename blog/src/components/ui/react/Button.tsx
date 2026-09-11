import type { ButtonHTMLAttributes } from 'react';

type Props = ButtonHTMLAttributes<HTMLButtonElement>;

/** Renders a consistently styled native button. */
export function Button({ className, ...props }: Props) {
	return <button className={['ui-button', className].filter(Boolean).join(' ')} {...props} />;
}
