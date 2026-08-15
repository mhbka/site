import type { ButtonHTMLAttributes } from 'react';

type Props = ButtonHTMLAttributes<HTMLButtonElement>;

export function Button({ className, ...props }: Props) {
	return <button className={['ui-button', className].filter(Boolean).join(' ')} {...props} />;
}
