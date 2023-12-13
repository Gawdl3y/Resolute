<template>
	<v-navigation-drawer :rail="!isExpanded" permanent width="180">
		<v-list nav>
			<v-tooltip text="Dashboard" :open-delay="500" :disabled="isExpanded">
				<template #activator="{ props }">
					<v-list-item
						title="Dashboard"
						:prepend-icon="mdiViewDashboard"
						to="/"
						v-bind="props"
					/>
				</template>
			</v-tooltip>

			<v-tooltip text="All Mods" :open-delay="500" :disabled="isExpanded">
				<template #activator="{ props }">
					<v-list-item
						title="All Mods"
						:prepend-icon="mdiPackageVariantClosedPlus"
						to="/mods"
						v-bind="props"
					/>
				</template>
			</v-tooltip>

			<v-tooltip text="Settings" :open-delay="500" :disabled="isExpanded">
				<template #activator="{ props }">
					<v-list-item
						title="Settings"
						:prepend-icon="mdiCog"
						to="/settings"
						v-bind="props"
					/>
				</template>
			</v-tooltip>
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
	mdiPackageVariantClosedPlus,
	mdiCog,
	mdiMenuClose,
	mdiMenuOpen,
} from '@mdi/js';

import sidebarBus from '../sidebar-bus';

const emit = defineEmits(['toggle']);
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
