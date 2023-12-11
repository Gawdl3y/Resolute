<template>
	<v-navigation-drawer :rail="isCollapsed" permanent width="180">
		<v-list nav>
			<v-tooltip text="Dashboard" :open-delay="500" :disabled="!isCollapsed">
				<template #activator="{ props }">
					<v-list-item
						title="Dashboard"
						:prepend-icon="mdiViewDashboard"
						to="/"
						v-bind="props"
					/>
				</template>
			</v-tooltip>

			<v-tooltip text="All Mods" :open-delay="500" :disabled="!isCollapsed">
				<template #activator="{ props }">
					<v-list-item
						title="All Mods"
						:prepend-icon="mdiPackageVariantClosedPlus"
						to="/mods"
						v-bind="props"
					/>
				</template>
			</v-tooltip>

			<v-tooltip text="Settings" :open-delay="500" :disabled="!isCollapsed">
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
					:text="isCollapsed ? 'Expand' : 'Collapse'"
					:open-delay="500"
					:disabled="!isCollapsed"
				>
					<template #activator="{ props }">
						<v-list-item
							:title="isCollapsed ? 'Expand' : 'Collapse'"
							:prepend-icon="isCollapsed ? mdiMenuClose : mdiMenuOpen"
							v-bind="props"
							@click="isCollapsed = !isCollapsed"
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

const isCollapsed = ref(true);
</script>
