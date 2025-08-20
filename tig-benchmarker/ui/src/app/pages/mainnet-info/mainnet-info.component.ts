import { Component, inject, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { CardModule } from 'primeng/card';
import { TableModule } from 'primeng/table';
import { TigApisService } from '../../services/tig-apis.service';

@Component({
  selector: 'app-mainnet-info',
  standalone: true,
  imports: [CommonModule, CardModule, TableModule],
  templateUrl: './mainnet-info.component.html',
  styleUrl: './mainnet-info.component.scss',
})
export class MainnetInfoComponent {
  tigService = inject(TigApisService);

  blockHeight = signal<number | null>(null);
  algorithms = signal<any[]>([]);

  ngOnInit() {
    this.tigService.mainnetInfo$.subscribe((data: any) => {
      if (data) {
        this.blockHeight.set(data.block_height);
        this.algorithms.set(data.algorithms);
      }
    });
    this.tigService.getMainnetInfo();
  }
}

