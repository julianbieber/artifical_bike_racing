FROM fedora:37

RUN dnf install xorg-x11-server-Xvfb util-linux alsa-lib-devel libXcursor libXrandr libXi python3 python3-pip -y

RUN python3 -m pip install jupyterlab

ADD assets/ /opt/assets
ADD artificial_bike_racing /opt/artifical_bike_racing

RUN ls /opt


ENTRYPOINT ["jupyter-lab", "--allow-root"]