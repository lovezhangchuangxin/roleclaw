<template>
  <section class="w-full">
    <WorldCardManager :world-cards="worldCards" :card-import-text="cardImportText"
      :card-export-path="cardExportPath" :ai-world-card-prompt="aiWorldCardPrompt"
      :ai-world-card-generating="aiWorldCardGenerating" :ai-generated-card="aiGeneratedWorldCard"
      :ai-world-card-stream-text="aiWorldCardStreamText" :ai-world-card-stream-parsed-ok="aiWorldCardStreamParsedOk"
      @update:card-import-text="cardImportText = $event" @update:card-export-path="cardExportPath = $event"
      @update:ai-world-card-prompt="aiWorldCardPrompt = $event" @import-card="importCardFromText"
      @export-card="exportCard" @duplicate-card="duplicateCard" @save-edited-card="saveEditedCard"
      @generate-card-by-ai="generateCardDraftWithAi" />
  </section>
</template>

<script setup lang="ts">
import WorldCardManager from "@/components/game/WorldCardManager.vue";
import { useGameAppContext } from "@/composables/useGameAppContext";

const {
  worldCards,
  cardImportText,
  cardExportPath,
  aiWorldCardPrompt,
  aiWorldCardGenerating,
  aiGeneratedWorldCard,
  aiWorldCardStreamText,
  aiWorldCardStreamParsedOk,
  importCardFromText,
  exportCard,
  duplicateCard,
  saveEditedCard,
  generateCardDraftWithAi,
} = useGameAppContext();
</script>
