import streamDeck, { action, KeyDownEvent, WillAppearEvent, WillDisappearEvent, SingletonAction } from "@elgato/streamdeck";

/**
 * Configuration for the Windows PC running the DeckRemote service
 */
interface PluginSettings {
    serverUrl?: string;
    key?: string;
}

/**
 * Default settings
 */
const DEFAULT_SETTINGS: PluginSettings = {
    serverUrl: "http://192.168.1.100:7394",
    key: "LWIN"
};

/**
 * Action to toggle key blocking on the remote Windows PC
 */
@action({ UUID: "com.deckremote.keyblock.toggle" })
class ToggleKeyBlockAction extends SingletonAction<PluginSettings> {
    private intervalId?: NodeJS.Timeout;
    private contexts: Map<string, PluginSettings> = new Map();

    /**
     * Called when the action appears on the Stream Deck
     */
    async onWillAppear(ev: WillAppearEvent<PluginSettings>): Promise<void> {
        const settings = { ...DEFAULT_SETTINGS, ...ev.payload.settings };
        this.contexts.set(ev.context, settings);
        
        // Update the initial state
        await this.updateState(ev.context, settings);
        
        // Start polling for state updates if not already polling
        if (!this.intervalId) {
            this.startPolling();
        }
    }

    /**
     * Called when the action disappears from the Stream Deck
     */
    async onWillDisappear(ev: WillDisappearEvent<PluginSettings>): Promise<void> {
        this.contexts.delete(ev.context);
        
        // Stop polling if no more actions are visible
        if (this.contexts.size === 0 && this.intervalId) {
            clearInterval(this.intervalId);
            this.intervalId = undefined;
        }
    }

    /**
     * Called when the action key is pressed
     */
    async onKeyDown(ev: KeyDownEvent<PluginSettings>): Promise<void> {
        const settings = { ...DEFAULT_SETTINGS, ...ev.payload.settings };
        
        try {
            const response = await fetch(`${settings.serverUrl}/keys/toggle`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ key: settings.key }),
            });

            if (response.ok) {
                const result = await response.json();
                
                // Update the state based on the response
                const state = result.blocked ? 1 : 0;
                await ev.action.setState(state);
                
                // Show success
                await ev.action.showOk();
            } else {
                await ev.action.showAlert();
            }
        } catch (error) {
            console.error('Failed to toggle key:', error);
            await ev.action.showAlert();
        }
    }

    /**
     * Start polling the server for state updates
     */
    private startPolling(): void {
        this.intervalId = setInterval(async () => {
            for (const [context, settings] of this.contexts.entries()) {
                await this.updateState(context, settings);
            }
        }, 2000); // Poll every 2 seconds
    }

    /**
     * Update the state of an action based on server state
     */
    private async updateState(context: string, settings: PluginSettings): Promise<void> {
        try {
            const response = await fetch(`${settings.serverUrl}/state`);
            
            if (response.ok) {
                const data = await response.json();
                const isBlocked = data.blockedKeys.includes(settings.key);
                const state = isBlocked ? 1 : 0;
                
                // Update the action state
                await streamDeck.actions.setState(state, context);
            }
        } catch (error) {
            // Silently fail - server might be offline
            console.error('Failed to fetch state:', error);
        }
    }
}

// Register the action and connect to Stream Deck
streamDeck.actions.registerAction(new ToggleKeyBlockAction());
streamDeck.connect();
