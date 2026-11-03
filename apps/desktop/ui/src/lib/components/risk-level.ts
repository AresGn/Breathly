import type { MessageKey } from '#lib/i18n/catalog.js';
import type { IconName } from './icons.js';

export type RiskLevel = 'safe' | 'review' | 'personal';

interface RiskPresentation {
	readonly label: MessageKey;
	/** Le CSS existant (`breathly.css`) nomme `verify` le niveau `review`. */
	readonly className: string;
	readonly icon: IconName;
}

export const RISK_PRESENTATIONS: Readonly<Record<RiskLevel, RiskPresentation>> = {
	safe: { label: 'risk.safe', className: 'risk--safe', icon: 'check' },
	review: { label: 'risk.review', className: 'risk--verify', icon: 'eye' },
	personal: { label: 'risk.personal', className: 'risk--personal', icon: 'user' }
};
