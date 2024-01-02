<template>
	<v-expansion-panels class="version-info-panels">
		<v-expansion-panel
			bg-color="rgba(var(--v-theme-on-surface), var(--v-activated-opacity))"
		>
			<v-expansion-panel-title>
				<h3 class="text-body-1">Artifacts</h3>
			</v-expansion-panel-title>
			<v-expansion-panel-text>
				<ModArtifactsTable :artifacts="version.artifacts" />
			</v-expansion-panel-text>
		</v-expansion-panel>

		<v-expansion-panel
			v-if="
				version.dependencies && Object.keys(version.dependencies).length > 0
			"
			bg-color="rgba(var(--v-theme-on-surface), var(--v-activated-opacity))"
		>
			<v-expansion-panel-title>
				<h3 class="text-body-1">Dependencies</h3>
			</v-expansion-panel-title>
			<v-expansion-panel-text>
				<ModDependenciesTable :dependencies="version.dependencies" />
			</v-expansion-panel-text>
		</v-expansion-panel>

		<v-expansion-panel
			v-if="version.conflicts && Object.keys(version.conflicts).length > 0"
			bg-color="rgba(var(--v-theme-on-surface), var(--v-activated-opacity))"
		>
			<v-expansion-panel-title>
				<h3 class="text-body-1">Conflicts</h3>
			</v-expansion-panel-title>
			<v-expansion-panel-text>
				<ModDependenciesTable :dependencies="version.conflicts" />
			</v-expansion-panel-text>
		</v-expansion-panel>
	</v-expansion-panels>
</template>

<script setup>
import ModArtifactsTable from './ModArtifactsTable.vue';
import ModDependenciesTable from './ModDependenciesTable.vue';

defineProps({ version: { type: Object, required: true } });
</script>

<style>
.version-info-panels .v-expansion-panel-text__wrapper {
	padding: 0 !important;
}

.version-info-panels .v-table {
	padding: 1em 0 0 0;
	background: rgba(var(--v-theme-surface), var(--v-medium-emphasis-opacity));
}
</style>
