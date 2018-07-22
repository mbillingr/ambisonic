#!/usr/bin/env python3
# -*- coding: utf-8 -*-

""" Convert SOFA HRIR files to the simple .hrir format for use by ambisonic.
"""

import sys
import os
import numpy as np
from netCDF4 import Dataset


virtual_speakers = np.array([[45, -30], [45, 30],
                             [135, -30], [135, 30],
                             [225, -30], [225, 30],
                             [315, -30], [315, 30]])
vs_rad = virtual_speakers * np.pi / 180


if __name__ == '__main__':
    if len(sys.argv) != 3:
        print("Usage: sofa2hrir <input.sofa> <output.hrir>")
        sys.exit(-1)
    _, infile, outfile = sys.argv
    
    infile = os.path.expanduser(infile)
    outfile = os.path.expanduser(outfile)
    
    rootgrp = Dataset(infile, 'r')
    
    pos = np.asarray(rootgrp.variables['SourcePosition'])
    ir = np.asarray(rootgrp.variables['Data.IR'])
    fs = rootgrp.variables['Data.SamplingRate'][0]
        
    # 6, 396, 1166
    C = np.array([[1 / np.sqrt(2),
                   np.cos(theta) * np.cos(phi),
                   np.sin(theta) * np.cos(phi),
                   np.sin(phi)] for theta, phi in vs_rad])
    
    CI = np.linalg.pinv(C)
    
    # find hrirs that closest match the speaker directions
    idx = [np.argmin(np.sum((pos[:, :2] - vs)**2, axis=1)) for vs in virtual_speakers]
    
    hrirs = ir[idx]
    
    w_hrir = (CI[0] * hrirs.T).sum(-1).T
    x_hrir = (CI[1] * hrirs.T).sum(-1).T
    y_hrir = (CI[2] * hrirs.T).sum(-1).T
    z_hrir = (CI[3] * hrirs.T).sum(-1).T
    
    coefs = np.stack([w_hrir, x_hrir, y_hrir, z_hrir]).transpose(1, 0, 2)
    
    with open(outfile, 'w') as f:
        print(fs, file=f)
        print(file=f)
        for side in [0, 1]:
            for idx in range(coefs.shape[2]):
                print(', '.join(str(c) for c in coefs[side, :, idx]), file=f)
            print(file=f)
                    