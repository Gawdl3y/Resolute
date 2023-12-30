<template>
	<v-navigation-drawer :rail="!isExpanded" permanent width="180">
		<v-list nav>
			<SidebarItem
				label="Dashboard"
				path="/"
				:icon="mdiViewDashboard"
				:expanded="isExpanded"
			/>
			<SidebarItem
				label="Mod Index"
				path="/mods"
				:icon="mdiPackageDown"
				:expanded="isExpanded"
			/>
			<SidebarItem
				label="Installed Mods"
				path="/mods/installed"
				:icon="mdiPackageCheck"
				:expanded="isExpanded"
			/>
			<SidebarItem
				v-if="settings.current.modAuthorTools"
				label="Author Tools"
				path="/author-tools"
				:icon="mdiToolbox"
				:expanded="isExpanded"
			/>
			<SidebarItem
				label="Settings"
				path="/settings"
				:icon="mdiCog"
				:expanded="isExpanded"
			/>
		</v-list>

		<template #append>
			<v-list nav>
				<v-tooltip
					:text="isExpanded ? 'Collapse' : 'Expand'"
					:open-delay="500"
					:disabled="isExpanded"
				>
					<template #activator="{ props }">
						<v-list-item
							:title="isExpanded ? 'Collapse' : 'Expand'"
							:prepend-icon="isExpanded ? mdiMenuOpen : mdiMenuClose"
							v-bind="props"
							@click="toggle"
						/>
					</template>
				</v-tooltip>
			</v-list>
		</template>
	</v-navigation-drawer>
</template>

<script setup>
import { ref } from 'vue';
import {
	mdiViewDashboard,
	mdiPackageDown,
	mdiPackageCheck,
	mdiToolbox,
	mdiCog,
	mdiMenuClose,
	mdiMenuOpen,
} from '@mdi/js';

import useSettings from '../composables/settings';
import sidebarBus from '../sidebar-bus';
import SidebarItem from './SidebarItem.vue';

const emit = defineEmits(['toggle']);
const settings = useSettings();
const isExpanded = ref(false);

/**
 * Toggles the collapsed state of the sidebar
 */
function toggle() {
	isExpanded.value = !isExpanded.value;
	emit('toggle', isExpanded.value);
	sidebarBus.emit('toggle', isExpanded.value);
}
</script>
