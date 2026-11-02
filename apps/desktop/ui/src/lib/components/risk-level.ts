import type { MessageKey } from '#lib/i18n/catalog.js';

export type RiskLevel = 'safe' | 'review' | 'personal';

export const RISK_LEVELS: Readonly<
	Record<RiskLevel, { label: MessageKey; className: string; icon: 'check' | 'eye' | 'user' }>
> = {
	safe: { label: 'risk.safe', className: 'risk--safe', icon: 'check' },
	review: { label: 'risk.review', className: 'risk--verify', icon: 'eye' },
	personal: { label: 'risk.personal', className: 'risk--personal', icon: 'user' }
};
