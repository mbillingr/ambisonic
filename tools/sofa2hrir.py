#!/usr/bin/env python3
# -*- coding: utf-8 -*-

""" Convert SOFA HRIR files to the simple .hrir format for use by ambisonic.
"""

import sys
import os
import numpy as np
from netCDF4 import Dataset


tetrahedron = np.array([[-np.sqrt(2/3), np.sqrt(2/9), -1/3],
                        [np.sqrt(2/3), np.sqrt(2/9), -1/3],
                        [0, -np.sqrt(8/9), -1/3],
                        [0, 0, 1]])

def calc_angles(cartesian):
    phi = np.arcsin(cartesian[:, 2])
    print(phi * 180 / np.pi)
    theta = -np.arctan2(cartesian[:, 0] , cartesian[:, 1])
    theta[theta < 0] = 2*np.pi + theta[theta < 0]
    print(theta * 180 / np.pi)
    
calc_angles(tetrahedron)


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
    
    C = np.concatenate([np.ones((4, 1)) / np.sqrt(2), tetrahedron], axis=1)
#    
    #CI = np.linalg.pinv(C)    
    CI = C.T
    
    # find hrirs that closest match the speaker directions
    
    #idx = [np.argmin(np.sum((pos[:, :2] - vs)**2, axis=1)) for vs in virtual_speakers]    
    #hrirs = ir[idx]
    
    ir *= 10  # adjust loundness ... this value is very ad-hoc, but seems to work for now
    
    hrirs = []
    hrirs.append(ir[289])   # 60 / -20
    hrirs.append(ir[1276])  # 300 / -20
    hrirs.append(ir[777])   # 180 / -20
    hrirs.append((ir[21] + ir[796]) / 2)  # 0 / 90; interpolate from 0/80 and 180/80
    
    with open(outfile, 'w') as f:
        print(fs, file=f)
        print(file=f)
        
        for i in range(CI.shape[1]):
            print(', '.join(str(ci) for ci in CI[:, i]), file=f)
            print(', '.join(str(ci) for ci in hrirs[i][0]), file=f)
            print(', '.join(str(ci) for ci in hrirs[i][1]), file=f)
            print(file=f)
                    