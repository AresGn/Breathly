export type IconName = 'check' | 'eye' | 'user' | 'alert' | 'info' | 'trash' | 'copy' | 'chevron';

export const ICON_PATHS: Readonly<Record<IconName, readonly string[]>> = {
	check: ['M3 12a9 9 0 1 0 18 0 9 9 0 1 0-18 0', 'm8 12.5 2.8 2.8L16 9.5'],
	eye: [
		'M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z',
		'M9.2 12a2.8 2.8 0 1 0 5.6 0 2.8 2.8 0 1 0-5.6 0'
	],
	user: ['M8.5 8a3.5 3.5 0 1 0 7 0 3.5 3.5 0 1 0-7 0', 'M5 20c0-3.9 3.1-7 7-7s7 3.1 7 7'],
	alert: ['M12 4 2.8 19.5h18.4z', 'M12 10v4.5M12 17.2h.01'],
	info: ['M3 12a9 9 0 1 0 18 0 9 9 0 1 0-18 0', 'M12 11v5.5M12 7.8h.01'],
	trash: ['M4 7h16M9.5 7V4.5h5V7M6.5 7l1 13h9l1-13M10 11v6M14 11v6'],
	copy: [
		'M8.5 10.5a2 2 0 0 1 2-2h7a2 2 0 0 1 2 2v7a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2z',
		'M15.5 8.5V6A1.5 1.5 0 0 0 14 4.5H6A1.5 1.5 0 0 0 4.5 6v8A1.5 1.5 0 0 0 6 15.5h2.5'
	],
	chevron: ['m6 9 6 6 6-6']
};
