import { message as alertMessage } from '@tauri-apps/plugin-dialog';
import {
	isPermissionGranted,
	requestPermission as requestNotificationPermission,
	sendNotification,
} from '@tauri-apps/plugin-notification';

import useSettings from './settings';

/**
 * Options for sending a notification
 * @typedef {Object} NotificationOptions
 * @property {string} [icon=null] Icon for the notification
 * @property {boolean} [alert=false] Whether to force using a basic alert message
 */

export function useNotifications() {
	const settings = useSettings();

	/**
	 * Sends a notification
	 * @param {'info'|'success'|'error'} type
	 * @param {string} title
	 * @param {string} msg
	 * @param {NotificationOptions} [options]
	 */
	async function message(
		type,
		title,
		msg,
		{ icon = null, alert = false } = {},
	) {
		if (
			alert ||
			!settings.current.nativeNotifications ||
			!(await requestPermission())
		) {
			await alertMessage(msg, {
				title,
				icon,
				type: type === 'success' ? 'info' : type,
			});
			return;
		}

		await sendNotification({ title, body: msg });
	}

	/**
	 * Sends an informational notification
	 * @param {string} title
	 * @param {string} msg
	 * @param {NotificationOptions} [options]
	 */
	async function info(title, msg, options) {
		await message('info', title, msg, options);
	}

	/**
	 * Sends a success notification
	 * @param {string} title
	 * @param {string} msg
	 * @param {NotificationOptions} [options]
	 */
	async function success(title, msg, options) {
		await message('info', title, msg, options);
	}

	/**
	 * Sends an error notification
	 * @param {string} title
	 * @param {string} msg
	 * @param {NotificationOptions} [options]
	 */
	async function error(title, msg, { icon = null, alert = true } = {}) {
		await message('error', title, msg, { icon, alert });
	}

	/**
	 * Requests permission to send system notifications
	 * @returns {bool} Whether permission was granted
	 */
	async function requestPermission() {
		let hasPermission = await isPermissionGranted();
		if (!hasPermission) {
			hasPermission = (await requestNotificationPermission()) === 'granted';
		}
		return hasPermission;
	}

	return { message, info, success, error, requestPermission };
}

export default useNotifications;
