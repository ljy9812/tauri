<script>
  import { TrayIcon } from '@tauri-apps/api/tray'
  import MenuBuilder from '../components/MenuBuilder.svelte'
  import { Menu, MenuItem, PredefinedMenuItem, CheckMenuItem, IconMenuItem } from '@tauri-apps/api/menu'

  let { onMessage } = $props()

  let icon = $state(null)
  let tooltip = $state(null)
  let title = $state(null)
  let iconAsTemplate = $state(false)
  let menuOnLeftClick = $state(true)
  let menuItems = $state([])
  let qoTitle = $state('Tauri API')
  let qoHeight = $state(300)
  let qoAbilityName = $state('TestTrayAbility')
  let qoModuleName = $state('entry_desktop')
  let testTray = $state(null)

  // Tauri 32x32 default icon
  const DEFAULT_ICON = 'data:image/png;base64,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';

  function onItemClick(detail) {
    onMessage(`Item ${detail.text} clicked`)
  }

  async function create() {
    // OHOS only allows one tray — remove any existing tray first
    await TrayIcon.removeById('tray-1').catch(() => {})
    await TrayIcon.removeById('manual-tray').catch(() => {})
    await new Promise(r => setTimeout(r, 500))

    try {
      await TrayIcon.new({
        id: 'manual-tray',
        icon: icon || DEFAULT_ICON,
        tooltip,
        title,
        iconAsTemplate,
        menuOnLeftClick,
        menu: await Menu.new({
          items: menuItems.map((i) => i.item)
        }),
        quickOperation: qoAbilityName
          ? {
              title: qoTitle,
              height: qoHeight,
              abilityName: qoAbilityName,
              moduleName: qoModuleName || undefined
            }
          : undefined,
        action: (event) => onMessage(event)
      })
      onMessage('Manual tray created')
    } catch (e) {
      onMessage(`Create manual tray failed: ${e}`)
    }
  }

  async function createFullTestTray() {
    // OHOS only allows one tray — remove any existing tray first
    await TrayIcon.removeById('tray-1').catch(() => {})
    await TrayIcon.removeById('full-test-tray').catch(() => {})
    await new Promise(r => setTimeout(r, 500))

    try {
      const normalItem = await MenuItem.new({ id: 'normal-item', text: 'Normal Item' })
      const checkItem = await CheckMenuItem.new({ id: 'check-item', text: 'Check Item', checked: false })
      const normalItem2 = await MenuItem.new({ id: 'normal-item-2', text: 'Another Normal' })
      const iconItem = await IconMenuItem.new({ id: 'icon-item', text: 'Icon Item', icon: DEFAULT_ICON })

      const predefinedItems = await Promise.all([
        PredefinedMenuItem.new({ item: 'Copy' }),
        PredefinedMenuItem.new({ item: 'Cut' }),
        PredefinedMenuItem.new({ item: 'Paste' }),
        PredefinedMenuItem.new({ item: 'SelectAll' }),
        PredefinedMenuItem.new({ item: 'Separator' }),
        PredefinedMenuItem.new({ item: 'Undo' }),
        PredefinedMenuItem.new({ item: 'Redo' }),
        PredefinedMenuItem.new({ item: 'Separator' }),
        PredefinedMenuItem.new({ item: 'Minimize' }),
        PredefinedMenuItem.new({ item: 'Maximize' }),
        PredefinedMenuItem.new({ item: 'Fullscreen' }),
        PredefinedMenuItem.new({ item: 'CloseWindow' }),
        PredefinedMenuItem.new({ item: 'Separator' }),
        PredefinedMenuItem.new({ item: 'Hide' }),
        PredefinedMenuItem.new({ item: 'ShowAll' }),
        PredefinedMenuItem.new({ item: 'BringAllToFront' }),
        PredefinedMenuItem.new({ item: 'Quit' }),
      ])

      const menu = await Menu.new({
        items: [
          normalItem,
          checkItem,
          iconItem,
          normalItem2,
          await PredefinedMenuItem.new({ item: 'Separator' }),
          ...predefinedItems,
        ]
      })

      testTray = await TrayIcon.new({
        id: 'full-test-tray',
        icon: DEFAULT_ICON,
        tooltip: 'Full Test Tray',
        title: 'Test',
        menu,
        quickOperation: qoAbilityName
          ? {
              title: qoTitle || 'Test Tray',
              height: qoHeight || 300,
              abilityName: qoAbilityName,
              moduleName: qoModuleName || undefined
            }
          : undefined,
        action: (event) => onMessage(`tray event: ${JSON.stringify(event)}`)
      })
      onMessage('Full test tray created')
    } catch (e) {
      onMessage(`Create full tray failed: ${e}`)
    }
  }

  async function removeTestTray() {
    const ids = ['tray-1', 'manual-tray', 'full-test-tray']
    for (const id of ids) {
      await TrayIcon.removeById(id).catch(() => {})
    }
    testTray = null
    onMessage('All tray icons removed')
  }
</script>

<div class="flex flex-col children:grow gap-2">
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Title"
      bind:value={title}
    />

    <input
      class="input grow"
      type="text"
      placeholder="Tooltip"
      bind:value={tooltip}
    />

    <label>
      <input type="checkbox" class="checkbox" bind:checked={menuOnLeftClick} />
      Menu on left click
    </label>
  </div>

  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Icon path"
      bind:value={icon}
    />

    <label>
      <input type="checkbox" class="checkbox" bind:checked={iconAsTemplate} />
      Icon as template
    </label>
  </div>

  <div class="flex children:grow">
    <MenuBuilder bind:items={menuItems} itemClick={onItemClick} />
  </div>

  <div class="flex gap-1 items-center">
    <span class="font-bold text-sm">QuickOperation (OHOS):</span>
  </div>
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="QO Title"
      bind:value={qoTitle}
    />
    <input
      class="input"
      type="number"
      placeholder="Height"
      bind:value={qoHeight}
      style="width: 80px"
    />
  </div>
  <div class="flex gap-1">
    <input
      class="input grow"
      type="text"
      placeholder="Ability Name"
      bind:value={qoAbilityName}
    />
    <input
      class="input grow"
      type="text"
      placeholder="Module Name"
      bind:value={qoModuleName}
    />
  </div>

  <div class="flex gap-1">
    <button class="btn" onclick={create} title="Creates the tray icon"
      >Create tray</button
    >
    <button class="btn" onclick={createFullTestTray} title="Create a tray with all item types"
      >Full Test Tray</button
    >
    <button class="btn" onclick={removeTestTray} title="Remove all tray icons"
      >Remove All Trays</button
    >
  </div>
</div>
