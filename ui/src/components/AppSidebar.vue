<template>
	<v-navigation-drawer :rail="!isExpanded" permanent width="180">
		<v-list nav>
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
				v-if="settings.current.console"
				label="Console"
				path="/log"
				:icon="mdiConsole"
				:expanded="isExpanded"
			/>
			<SidebarItem
				label="About"
				path="/about"
				:icon="mdiInformation"
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
				<SimpleTooltip
					v-slot="{ props }"
					:text="isExpanded ? 'Collapse' : 'Expand'"
					:disabled="isExpanded"
				>
					<v-list-item
						v-bind="props"
						:title="isExpanded ? 'Collapse' : 'Expand'"
						:prepend-icon="isExpanded ? mdiMenuOpen : mdiMenuClose"
						@click="toggle"
					/>
				</SimpleTooltip>
			</v-list>
		</template>
	</v-navigation-drawer>
</template>

<script setup>
import { ref } from 'vue';
import {
	mdiPackageDown,
	mdiPackageCheck,
	mdiToolbox,
	mdiConsole,
	mdiInformation,
	mdiCog,
	mdiMenuClose,
	mdiMenuOpen,
} from '@mdi/js';

import useSettings from '../composables/settings';
import sidebarBus from '../sidebar-bus';
import SidebarItem from './SidebarItem.vue';
import SimpleTooltip from './SimpleTooltip.vue';

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
