import React, { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

interface PlanItem {
    id: string;
    title: string;
    description?: string;
    status: 'Pending' | 'InProgress' | 'Completed' | 'Failed' | 'Blocked';
    created_at: string;
}

export function PlanView() {
    const [plan, setPlan] = useState<PlanItem[]>([]);

    useEffect(() => {
        // Initial fetch
        invoke<PlanItem[]>('get_current_plan').then(setPlan).catch(console.error);

        // Listen for updates
        const unlisten = listen<PlanItem[]>('plan-updated', (event) => {
            setPlan(event.payload);
        });

        return () => {
            unlisten.then(f => f());
        };
    }, []);

    return (
        <div className="p-4 space-y-4">
            <h2 className="text-xl font-bold">Agent Plan</h2>
            <div className="space-y-2">
                {plan.map(item => (
                    <div key={item.id} className="border p-3 rounded-lg flex justify-between items-center bg-card">
                        <div>
                            <h3 className="font-medium">{item.title}</h3>
                            {item.description && <p className="text-sm text-muted-foreground">{item.description}</p>}
                        </div>
                        <div className={`px-2 py-1 rounded text-xs ${getStatusColor(item.status)}`}>
                            {item.status}
                        </div>
                    </div>
                ))}
                {plan.length === 0 && (
                    <p className="text-muted-foreground text-center py-8">No active plan items.</p>
                )}
            </div>
        </div>
    );
}

function getStatusColor(status: string) {
    switch (status) {
        case 'Pending': return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-100';
        case 'InProgress': return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-100';
        case 'Completed': return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-100';
        case 'Failed': return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-100';
        default: return 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-100';
    }
}
