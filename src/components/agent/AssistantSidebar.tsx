import React, { useState, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';

interface Suggestion {
    id: string;
    text: string;
    type: string;
    persona_name: string;
    persona_role: string;
}

export function AssistantSidebar() {
    const [suggestions, setSuggestions] = useState<Suggestion[]>([]);

    useEffect(() => {
        const unlisten = listen<Suggestion>('new-suggestion', (event) => {
            setSuggestions(prev => [event.payload, ...prev].slice(0, 10));
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    const [mode, setMode] = useState('Technical');

    const handleModeChange = (newMode: string) => {
        setMode(newMode);
        invoke('set_meeting_mode', { mode: newMode });
    };

    return (
        <div className="h-full flex flex-col border-l w-80 bg-background">
            <div className="p-4 border-b flex justify-between items-center">
                <div>
                    <h2 className="font-bold">Active Team</h2>
                    <p className="text-xs text-muted-foreground">Live suggestions</p>
                </div>
                <select
                    value={mode}
                    onChange={(e) => handleModeChange(e.target.value)}
                    className="text-xs border rounded p-1 bg-background"
                >
                    <option value="Discovery">Discovery</option>
                    <option value="Technical">Technical</option>
                    <option value="Review">Review</option>
                </select>
            </div>
            <div className="flex-1 overflow-y-auto p-4 space-y-3">
                {suggestions.map(s => (
                    <div key={s.id} className="p-3 rounded-lg border bg-card shadow-sm animate-in slide-in-from-right-2">
                        <div className="flex items-center gap-2 mb-1">
                            <div className="w-6 h-6 rounded-full bg-primary/10 flex items-center justify-center text-xs font-bold text-primary">
                                {s.persona_name[0]}
                            </div>
                            <div className="flex flex-col flex-1">
                                <span className="text-xs font-bold">{s.persona_name}</span>
                                <span className="text-[10px] text-muted-foreground uppercase">{s.persona_role}</span>
                            </div>
                            <button
                                onClick={() => invoke('speak_suggestion', { text: s.text, personaRole: s.persona_role })}
                                className="p-1 hover:bg-muted rounded text-muted-foreground hover:text-foreground transition-colors"
                                title="Allow to speak"
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><line x1="12" y1="19" x2="12" y2="22" /></svg>
                            </button>
                        </div>
                        <p className="text-sm mt-1">{s.text}</p>
                    </div>
                ))}
                {suggestions.length === 0 && (
                    <div className="text-center py-10 text-muted-foreground text-sm">
                        Listening to meeting...
                    </div>
                )}
            </div>
        </div>
    );
    switch (type) {
        case 'question': return 'bg-blue-500';
        case 'technical': return 'bg-purple-500';
        case 'edge-case': return 'bg-orange-500';
        default: return 'bg-gray-500';
    }
}
