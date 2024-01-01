<template>
	<v-tooltip :open-delay="500">
		<template #activator="{ props: tooltipProps }">
			<div class="d-flex h-100 position-relative">
				<v-scroll-y-transition>
					<div
						:key="mod.versionTextClass ?? 'plain'"
						v-bind="tooltipProps"
						class="d-flex gc-2 align-center justify-space-between position-absolute h-100"
						:class="mod.versionTextClass"
					>
						{{ mod.installedVersion?.semver ?? mod.latestVersion.semver }}

						<v-icon v-if="mod.hasUpdate" :icon="mdiAlert" size="small" />
						<v-icon
							v-else-if="mod.installedVersion"
							:icon="mdiCheckCircle"
							size="small"
						/>
					</div>
				</v-scroll-y-transition>
			</div>
		</template>

		<dl class="d-flex flex-wrap" style="width: 7.25em">
			<dt class="w-50">Installed:</dt>
			<dd class="w-50 ms-auto text-end">
				{{ mod.installedVersion?.semver ?? 'None' }}
			</dd>
			<dt class="w-50">Latest:</dt>
			<dd class="w-50 ms-auto text-end">
				{{ mod.latestVersion.semver }}
			</dd>
		</dl>
	</v-tooltip>
</template>

<script setup>
import { mdiCheckCircle, mdiAlert } from '@mdi/js';

import { ResoluteMod } from '../../structs/mod';

defineProps({ mod: { type: ResoluteMod, required: true } });
</script>
