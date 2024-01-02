<template>
	<div class="d-flex flex-wrap ga-2">
		<v-chip color="primary" variant="flat">{{ mod.category }}</v-chip>

		<v-chip
			v-for="flag of mod.flags"
			:key="flag"
			:color="flagColor(flag)"
			variant="flat"
		>
			{{ flag[0].toUpperCase() }}{{ flag.substring(1) }}
		</v-chip>

		<v-chip v-if="!mod.platforms" color="indigo" variant="flat">
			All Platforms
		</v-chip>
		<v-chip
			v-for="platform of mod.platforms"
			v-else
			:key="platform"
			:color="platformColor(platform)"
			variant="flat"
		>
			{{ platformLabel(platform) }}
		</v-chip>

		<v-chip v-for="tag of mod.tags" :key="tag" variant="flat">
			{{ tag }}
		</v-chip>
	</div>
</template>

<script setup>
defineProps({ mod: { type: Object, required: true } });

/**
 * Determines the color to use for a given flag
 * @param {string} flag
 */
function flagColor(flag) {
	switch (flag) {
		case 'deprecated':
			return 'error';
		case 'final':
			return 'warning';
		default:
			return 'secondary';
	}
}

/**
 * Determines the label to use for a given platform
 * @param {string} platform
 */
function platformLabel(platform) {
	switch (platform) {
		case 'headless':
			return 'Headless';
		case 'windows':
			return 'Windows';
		case 'linux':
			return 'Linux';
		case 'linux-native':
			return 'Linux (native)';
		case 'linux-wine':
			return 'Linux (Wine)';
		case 'android':
			return 'Android';
		default:
			return 'Other platform';
	}
}

/**
 * Determines the color to use for a given platform
 * @param {string} platform
 */
function platformColor(platform) {
	switch (platform) {
		case 'headless':
			return 'blue-grey';
		case 'windows':
			return 'blue';
		case 'linux':
		case 'linux-native':
			return 'teal';
		case 'linux-wine':
			return 'purple';
		case 'android':
			return 'green';
		default:
			return 'orange';
	}
}
</script>
