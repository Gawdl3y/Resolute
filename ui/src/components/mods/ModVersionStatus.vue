<template>
	<v-tooltip :open-delay="500">
		<template #activator="{ props: tooltipProps }">
			<div class="d-flex h-100 position-relative">
				<v-scroll-y-transition>
					<div
						v-bind="tooltipProps"
						:key="mod.versionTextClass ?? 'plain'"
						class="d-flex gc-2 align-center justify-space-between position-absolute h-100"
						:class="mod.versionTextClass"
					>
						<span :class="{ 'text-decoration-line-through': mod.isDeprecated }">
							{{ (mod.installedVersion ?? mod.latestVersion).label }}
						</span>

						<v-icon v-if="mod.hasUpdate" :icon="mdiAlert" size="small" />
						<v-icon
							v-else-if="mod.isUnrecognized"
							:icon="mdiHelpCircle"
							size="small"
						/>
						<v-icon
							v-else-if="mod.installedVersion"
							:icon="mdiCheckCircle"
							size="small"
						/>
					</div>
				</v-scroll-y-transition>
			</div>
		</template>

		<dl class="d-flex flex-wrap" :style="`width: ${tooltipWidth}`">
			<dt class="w-50">Installed:</dt>
			<dd class="w-50 ms-auto text-end">
				{{ mod.installedVersion?.label ?? 'None' }}
			</dd>
			<dt class="w-50">Latest:</dt>
			<dd class="w-50 ms-auto text-end">{{ mod.latestVersion.label }}</dd>
		</dl>
	</v-tooltip>
</template>

<script setup>
import { computed } from 'vue';
import { mdiCheckCircle, mdiAlert, mdiHelpCircle } from '@mdi/js';

import { ResoluteMod } from '../../structs/mod';

const props = defineProps({ mod: { type: ResoluteMod, required: true } });

const tooltipWidth = computed(() => {
	const mod = props.mod;
	const anyUnrecognized =
		mod.installedVersion?.isUnrecognized || mod.latestVersion.isUnrecognized;
	if (anyUnrecognized) return '8.75em';
	return '8.25em';
});
</script>
