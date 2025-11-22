'use client';

import { Search } from 'lucide-react';
import { Input } from '@/components/ui/input';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';

interface SearchBarProps {
  searchQuery: string;
  filterType: 'all' | 'active' | 'muted';
  sessionCounts: {
    all: number;
    active: number;
    muted: number;
  };
  onSearchChange: (query: string) => void;
  onFilterChange: (filter: 'all' | 'active' | 'muted') => void;
}

export function SearchBar({
  searchQuery,
  filterType,
  sessionCounts,
  onSearchChange,
  onFilterChange,
}: SearchBarProps) {
  return (
    <div className="mb-6">
      <h2 className="text-xl font-semibold mb-4">程式音量</h2>

      <div className="flex flex-col md:flex-row gap-4 mb-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4" />
          <Input
            type="text"
            placeholder="搜尋程式名稱..."
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            className="pl-10"
          />
        </div>

        <Tabs value={filterType} onValueChange={(value) => onFilterChange(value as 'all' | 'active' | 'muted')}>
          <TabsList>
            <TabsTrigger value="all">
              全部 ({sessionCounts.all})
            </TabsTrigger>
            <TabsTrigger value="active">
              開啟 ({sessionCounts.active})
            </TabsTrigger>
            <TabsTrigger value="muted">
              靜音 ({sessionCounts.muted})
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>
    </div>
  );
}
