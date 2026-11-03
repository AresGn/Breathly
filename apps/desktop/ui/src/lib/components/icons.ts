export type IconName = 'check' | 'eye' | 'user' | 'alert' | 'info';

export const ICON_PATHS: Readonly<Record<IconName, readonly string[]>> = {
	check: ['M3 12a9 9 0 1 0 18 0 9 9 0 1 0-18 0', 'm8 12.5 2.8 2.8L16 9.5'],
	eye: [
		'M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z',
		'M9.2 12a2.8 2.8 0 1 0 5.6 0 2.8 2.8 0 1 0-5.6 0'
	],
	user: ['M8.5 8a3.5 3.5 0 1 0 7 0 3.5 3.5 0 1 0-7 0', 'M5 20c0-3.9 3.1-7 7-7s7 3.1 7 7'],
	alert: ['M12 4 2.8 19.5h18.4z', 'M12 10v4.5M12 17.2h.01'],
	info: ['M3 12a9 9 0 1 0 18 0 9 9 0 1 0-18 0', 'M12 11v5.5M12 7.8h.01']
};
