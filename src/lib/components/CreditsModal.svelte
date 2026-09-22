<script lang="ts">
  import Modal from "./Modal.svelte";

  interface Props {
    onclose: () => void;
  }

  let { onclose }: Props = $props();
</script>

<Modal title="Credits" {onclose}>
  <ul class="credits">
    <li>
      <span class="caps">Original idea</span>
      <span class="who">Adrián Rascón</span>
    </li>
    <li>
      <span class="caps">Developed by</span>
      <span class="who">Diego Rascón<br />Adrián Rascón</span>
    </li>
  </ul>
</Modal>

<style>
  .credits {
    display: flex;
    flex-direction: column;
    gap: 18px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  li {
    display: grid;
    grid-template-columns: 110px 1fr;
    gap: 16px;
    align-items: baseline;
  }

  .who {
    font-size: 14.5px;
    font-weight: 500;
    line-height: 1.6;
  }
</style>
